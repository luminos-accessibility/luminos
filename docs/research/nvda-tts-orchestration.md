# Speech Orchestration & Command System for Screen Readers

### A methodology reference distilled from NVDA, for reimplementation in any language or operating system

---

## Preface — scope and intent

This document captures the **algorithms, data structures, and design methodology** that a screen reader uses to turn application events and keystrokes into well-timed speech (and braille) output. It is distilled from the NVDA screen reader but is deliberately written to be **programming-language- and operating-system-agnostic**. It contains no source code and avoids platform-specific APIs; it describes *what* the system does and *why*, so the same intelligence can be reimplemented on a different stack.

**In scope:** the speech-sequence intermediate representation, the priority/interruption scheduler, continuous-reading and fluency techniques, the text-to-speech-sequence processing pipeline (symbols, dictionaries, language, normalization, markup), the synthesizer-driver abstraction and its feedback contract, and the command/gesture (hotkey) system.

**Explicitly out of scope:** the actual *speech synthesis* (waveform generation). A screen reader of this design does **not** synthesize audio itself — it delegates to external TTS engines. The intelligence here is *orchestration*, not digital signal processing. Where the reference engine is named (a formant synthesizer, etc.), treat it as a replaceable backend reached through the driver contract in §6.

**The one idea to internalize first:** the system is driven by a **feedback clock**. The synthesizer reports *index marks* as audio physically plays; those reports — not timers, not guesses — tell the orchestrator when an utterance finished, when to advance the reading cursor, and when to push the next piece of speech. Almost every technique below hangs off that clock.

### Table of contents
1. System overview and the central design principle
2. The Speech Sequence model (the intermediate representation)
3. The Speech Manager (scheduling, priority, interruption)
4. Continuous reading and fluency techniques
5. The text-to-speech-sequence processing pipeline
6. The synthesizer-driver abstraction
7. The command and gesture (hotkey) system
8. Implementation considerations for a different OS/language
- Appendix A — Full command reference (desktop / laptop / touch bindings)
- Appendix B — Browse-mode single-letter quick navigation
- Appendix C — Review-cursor and object-navigation command sets
- Appendix D — Braille display gesture model
- Appendix E — Touchscreen gesture model

---

## 1. System overview and the central design principle

### 1.1 The pipeline

A single conceptual pipeline turns a *cause* (a focus change, a caret movement, a keystroke, a content read request) into *output*:

```
cause ─▶ what-to-say logic ─▶ text processing ─▶ Speech Sequence ─▶ Scheduler ─▶ Driver ─▶ engine ─▶ audio
                                                                          ▲                                  │
                                                                          └──────── index-mark feedback ─────┘
```

- **What-to-say logic** decides *which* properties of an object or document to report, in what order, and what to suppress (covered as repetition-suppression in §4.3 and object/field presentation in §4.3/§5).
- **Text processing** (§5) transforms human-readable text into a normalized, symbol-expanded, language-tagged form.
- **The Speech Sequence** (§2) is the intermediate representation: a flat stream interleaving text with control commands.
- **The Scheduler / Speech Manager** (§3) decides *when* each fragment is spoken, handles priority and interruption, and tracks completion.
- **The Driver** (§6) adapts the sequence to a specific engine and relays the engine's progress back as the feedback clock.

### 1.2 Design principles that recur throughout

1. **Index-mark feedback as the master clock.** The orchestrator inserts numbered marks into the stream. The engine signals when each mark is reached *in played audio*. Completion, cursor advancement, and callbacks are all driven by these signals, never by wall-clock timing. This makes the system robust to varying speech rates and engine latency.
2. **An intermediate representation decouples policy from engines.** All higher layers produce one neutral Speech Sequence; each engine driver knows how to consume it. Adding an engine never touches the orchestration logic.
3. **Capability negotiation with graceful degradation.** Drivers advertise which commands and settings they support. The orchestrator adapts (e.g. spelling out a phoneme as fallback text, announcing that a language is unsupported) rather than failing.
4. **Layered, overridable command targets.** Behavior is resolved through a chain (global → application-specific → document/interceptor → focused object), so app-specific and content-specific behavior can override defaults without modifying them. The same layering applies to object classes, event handlers, and gesture bindings.
5. **A single unified input model.** Keyboard, braille keys, and touch gestures all become the same kind of abstract "gesture" object and resolve through the same map. Adding an input modality does not fork the command system.
6. **Single-threaded core fed by a queue.** Asynchronous callbacks from the OS, the engine, and input devices are marshalled onto one core thread via a queue and drained by a periodic pump, eliminating races without locks in the hot path.

---

## 2. The Speech Sequence model (the intermediate representation)

### 2.1 Structure

A **Speech Sequence** is an ordered, flat list whose elements are either **text fragments** or **command objects**. Commands are interleaved positionally with text, so prosody, language, pauses, and markers apply from their position onward (or at their exact point). There is no nesting; generators that yield sub-sequences are flattened before scheduling.

The sequence is the *only* thing the scheduler and drivers consume. Everything upstream — object inspection, text processing, formatting — exists to produce sequences.

### 2.2 Command taxonomy

Commands fall into two families with very different lifecycles. **Synth-facing** commands are forwarded to the engine and affect how text is spoken. **Control** commands are consumed by the scheduler and never reach the engine; they drive timing, callbacks, and structure.

**Synth-facing commands**

| Command | Meaning |
|---|---|
| Language change | Switch the speaking language/voice from this point; a "default/reset" form returns to the base language. |
| Character-mode | Toggle character-by-character pronunciation (spelling) on/off. |
| Pitch / Rate / Volume | Adjust prosody, expressed as either an absolute offset or a multiplier relative to the user's base setting. |
| Break | Insert a pause of a given duration. |
| Phoneme | Speak a precise phonetic (IPA) pronunciation, with plain-text fallback if the engine cannot honor it. |

**Control commands (scheduler-only)**

| Command | Meaning |
|---|---|
| Index | A numbered position marker. The fundamental unit of the feedback clock (§3.2). |
| End-utterance | An explicit boundary that forces the scheduler to close the current utterance and start a new one. |
| Callback | Run an arbitrary action *when speech reaches this point in played audio* (used for cursor advancement, chaining continuous reads, etc.). Must return quickly. |
| Beep / Wave | Emit a tone or play an audio asset at this point in the stream (e.g. to indicate capitals, progress, or errors). |
| Profile-trigger | Enter/exit a configuration profile at this point (e.g. switch voice settings for "say all" or for a particular language/app). |
| Cancellable-speech marker | Associates the following speech with a validity check so it can be dropped if it becomes stale before it is spoken (e.g. focus moved again). |

### 2.3 Why offsets *and* multipliers for prosody

Prosody commands support both an absolute offset and a multiplier because two needs coexist: transient, content-driven emphasis (e.g. raise pitch for capitals) composes naturally as a multiplier on top of whatever base rate/pitch the user has chosen, while certain features need a fixed absolute change. Keeping both forms in the IR lets the driver translate either into the engine's native parameter range.

---

## 3. The Speech Manager (scheduling, priority, interruption)

The Speech Manager is the heart of the system: a **priority-queued, index-clocked scheduler** that decides what is spoken, when, and what happens when something more urgent arrives.

### 3.1 Priority model

Speech is queued at one of three priority levels:

| Priority | Semantics |
|---|---|
| **Normal** | Append behind everything already queued at normal priority. |
| **Next** | Speak after the *current* utterance finishes, ahead of pending normal speech. |
| **Now** | Interrupt immediately: cancel current audio, speak this at once, then **resume** the lower-priority speech that was interrupted. |

The defining behavior is that **Now does not discard lower-priority speech — it preempts and later resumes it.** Each priority level keeps its *own* pending queue and its own state, so an interrupting announcement (say, a focus change) can barge in over a long document read, and the read continues afterward from where it was preempted.

### 3.2 The index/callback clock — detecting completion

This is the mechanism everything else relies on:

1. The scheduler guarantees **every utterance ends with an index mark**. If the last element before an utterance boundary is not already an index, one is appended.
2. Every Callback control command is converted, at schedule time, into (a) an index mark inserted into the outgoing stream and (b) an entry in a *pending-callbacks* map keyed by that index number.
3. Index numbers are drawn from a rolling counter (recycled within a bounded range).
4. When the engine reports "index N reached" — which the driver raises **as that audio segment actually finishes playing** (§6.4) — the scheduler:
   - removes the finished sequence(s) up to N from the in-flight set,
   - runs any callback registered for N,
   - if N was an utterance-ending index, pushes the next utterance.

Because progress is measured by *played audio*, the orchestrator stays perfectly in step with what the user is actually hearing, at any speech rate, without timers.

### 3.3 Utterance assembly and parameter replay

The scheduler assembles each **utterance** by concatenating pending fragments until it hits an end-utterance boundary, a callback split point, or a profile-trigger boundary.

A subtle but essential technique is **parameter replay**: the scheduler tracks which prosody/language parameters are currently non-default. When an utterance is preempted (priority Now) and a lower-priority utterance later resumes, the tracked parameter commands are **re-emitted at the head of the resumed utterance**. Without this, an interruption would reset pitch/rate/language mid-content. The same applies when a long sequence is split into multiple engine utterances — the active parameters are carried across the split.

### 3.4 Cancellation semantics

Two distinct cancellation paths exist:

- **User/priority cancellation:** a Now-priority arrival (or an explicit "stop speech") cancels the engine and clears the in-flight set for the affected priority, then pushes the next eligible speech.
- **Staleness cancellation:** speech can be tagged with a *validity check* (the cancellable-speech marker). Before a tagged segment is handed to the engine, its validity is tested; if the originating context is gone (e.g. focus already moved on again), the segment and everything up to its index are dropped. This prevents the reader from announcing state that is already obsolete by the time the engine is free — a common source of "laggy, talking about the wrong thing" behavior in naive implementations.

### 3.5 Per-priority queue state and profile synchronization

Each priority level owns a queue object holding its pending sequences, its set of entered configuration-profile triggers, and its parameter tracker. When the active priority changes, the scheduler must keep the engine's *configuration* consistent with the speech being resumed:

1. Exit the profile triggers belonging to the outgoing queue.
2. If profile/voice switches are pending, **wait for the engine to drain** so the switch does not corrupt in-flight audio.
3. Restore the profile triggers belonging to the queue being resumed.

This is what allows, for example, a "say all" read to run under a distinct voice profile and have normal announcements interleave correctly.

---

## 4. Continuous reading and fluency techniques

Reading large content aloud smoothly is harder than it looks. Three cooperating techniques produce natural, gap-free continuous reading.

### 4.1 Say-All (continuous read) — generator + callback chaining

"Say All" reads from the cursor to the end of a document continuously. The methodology:

1. A **reader** (a stateful generator) yields the next *reading chunk* of text from the content model (by reading-unit: roughly a line/sentence-sized unit).
2. For each chunk, a **callback control command is prepended** that, when reached in audio, updates the visible/review cursor to that chunk's position and requests the *next* chunk.
3. The chunk's speech is produced and handed to the speak-without-pauses buffer (§4.2), then to the scheduler.
4. When audio playback reaches the prepended callback, the cursor is advanced and the reader is pumped again — **chaining** one chunk to the next via the feedback clock rather than pre-rendering everything.
5. **Look-ahead buffering:** several chunks (a small bounded number) may be requested ahead of playback so the engine is never starved, but the buffer is capped to bound memory and latency.
6. **Boundary handling:** at the end of the content, if the container supports advancing (e.g. turning a page / loading more), a callback requests the next region and continues; otherwise the read ends.

The cursor therefore tracks the audio, the read is interruptible at any moment (the chain simply stops being pumped), and resuming is natural.

### 4.2 Speak-without-pauses — sentence-boundary buffering

Engines insert a small pause at the end of each utterance. If content is fed fragment-by-fragment (as say-all does), the listener hears choppy, stop-start speech. The fix:

1. Maintain a **pending buffer** across calls.
2. On each new fragment, scan from the **end backward** for the last natural sentence/clause boundary (terminal punctuation followed by whitespace or end of input).
3. Everything up to that boundary is flushed as a complete utterance; everything after the boundary is **retained** in the pending buffer to be prepended to the next fragment.
4. Active language context (the most recent language-change command) is **replayed** with the retained text so a buffered fragment keeps speaking in the correct language across the call boundary.
5. Explicit end-utterance commands force a flush at their position.

The result is that utterance boundaries fall at *linguistic* boundaries, not arbitrary fragment edges, so continuous reading sounds like prose.

### 4.3 Repetition suppression and contextual presentation

Speaking everything on every event is exhausting and slow. The system speaks **only what changed** and only what is contextually new.

- **Property caching with delta detection.** Each object caches its last-spoken properties. On a *change* event, new property values are compared to the cache and **unchanged properties are dropped**. For multi-valued properties such as states, the system computes a delta — newly added states vs. newly removed states — and announces the difference (e.g. "checked", "not pressed") rather than the whole state set.
- **Positional suppression.** Redundant positional context (e.g. a table cell's row number when only the column changed) is suppressed by comparing against the previously announced coordinates.
- **Field-stack tracking for structured content.** When reading rich text, the reader maintains two stacks across calls: a stack of nested **control fields** (containers: lists, tables, dialogs, regions) and a stack of **format fields** (bold, italic, font, color, language). Comparing the new stacks to the previous ones yields exactly the *entered* and *exited* containers and the *changed* formatting, so the reader announces "list", "out of list", "bold" at the right moments and stays silent otherwise. The reason for an event (focus, caret move, say-all, query, mouse) tunes which transitions are spoken.

### 4.4 The extension / filter pipeline

The pipeline is **interceptable** so behavior can be extended without forking it. Before a sequence is scheduled it passes through:

- a **sequence filter** stage — registered handlers may rewrite the sequence (insert, remove, or transform elements); and
- **notification hooks** — pre-speech, pre-queued, pre-cancel, post-pause — that let extensions observe or react.

Notably, *automatic language announcement* (§5.5) is itself implemented as a registered sequence-filter handler, demonstrating that even core features compose cleanly through this seam.

---

## 5. The text-to-speech-sequence processing pipeline

Before text becomes a Speech Sequence it passes through an ordered series of transforms. **Order matters** — applying them in the wrong order produces wrong pronunciations.

### 5.1 Canonical order

1. **Speech-dictionary substitution** (user/voice/temporary/built-in replacements).
2. **Symbol/punctuation processing** (expand symbols to words per the active verbosity level).
3. **Whitespace normalization** (collapse control characters to spaces).
4. **Unicode normalization** (optional; canonicalize decorative/compatibility characters).
5. **Trimming** (strip redundant surrounding whitespace).
6. **Language tagging** (insert language-change commands; optionally announce language names).
7. **Character/spelling handling** (when spelling, wrap in character-mode and attach per-character descriptions).
8. **Markup generation** (for engines that consume markup; §5.7).

### 5.2 Symbol and punctuation pronunciation

A configurable **verbosity level** governs how many symbols are spoken:

| Level | Behavior |
|---|---|
| None | Speak no symbols (punctuation silent, but may still affect prosody). |
| Some | Speak only the most essential symbols. |
| Most | Speak most symbols. |
| All | Speak every defined symbol. |
| Character | Character-by-character mode (used when spelling). |

Each symbol definition carries: an **identifier** (the character or pattern), a **replacement** (spoken text), a **level** (the minimum verbosity at which its replacement is spoken), and a **preserve** rule.

**Preserve rule** controls whether the original symbol is *also* sent to the engine alongside its spoken name — important because the symbol itself may carry prosodic meaning (a period ends a sentence; a comma induces a slight pause) even when its name is or isn't spoken. The three modes are: never preserve, always preserve, and preserve only when the symbol's name is *not* being spoken (i.e. below its level).

**The processor is a single compiled pattern matcher**, not a loop of replacements, for performance and correct precedence:

1. **Complex symbols** (defined by regular-expression patterns, e.g. "a period that ends a sentence" vs. a decimal point) are matched **first** and with highest precedence; they may reference capture groups in their replacement.
2. **Repeated-character runs** are detected and collapsed into a spoken count (e.g. "10 stars") when verbosity warrants; otherwise handled per the preserve rule.
3. **Simple single/multi-character symbols** are compiled together — single characters into a character class, multi-character symbols ordered **longest-first** so greedy matching prefers the most specific symbol.

All of these are combined into one matcher with named alternatives, applied in a single pass. The replacement is emitted surrounded by spaces so adjacent words and digits do not run together.

### 5.3 Symbol/locale data model

Symbol definitions are **layered by source** with inheritance, resolved into one computed table per locale:

- user-defined symbols (highest precedence),
- the locale's built-in symbols,
- a base-language fallback (e.g. the generic language behind a regional variant),
- and a universal fallback.

Per field (replacement, level, preserve, display name), the value from the highest-precedence source that defines it wins; missing fields take defaults (level = all, preserve = never). Definitions are stored as simple tab-delimited records with escape conventions for control characters — a format trivial to reproduce in any language. Separate analogous tables hold **character descriptions** (phonetic/word descriptions used when spelling, e.g. the radio-alphabet) and an optional emoji/symbol-name table sourced from common locale data.

### 5.4 Character descriptions and spelling

When spelling text (character mode), each character may be expanded to a description for disambiguation (e.g. distinguishing similar-sounding letters). Lookup follows a **locale → base-language → universal** fallback chain. Multi-codepoint characters (combining sequences, surrogate-pair emoji) are split on true character boundaries — not code units — so spelling handles the full Unicode range correctly. Capitals can be signaled by a pitch change, a spoken word, or a beep, depending on configuration.

### 5.5 Automatic language switching — declarative, not statistical

A crucial design decision: **language is determined from authoritative metadata, not guessed from the text.** The reader does *not* run statistical language detection. Instead, the *source* of the content supplies the language — document markup language attributes, rich-text language runs, editor language fields — and the reader emits a language-change command for each run. Implications for a reimplementation:

- Your content adapters (the equivalent of accessibility-tree/document readers) are responsible for surfacing language-per-run.
- A **dialect** policy decides whether regional variants are distinguished or collapsed to the base language (e.g. treat all English regions as "English" unless dialect switching is enabled).
- Optionally the *name* of the new language is announced (spoken in the base language) when language reporting is on; if the active engine/voice cannot speak the requested language, that condition is detected and optionally signaled rather than silently mispronounced.

### 5.6 Unicode normalization and decorative characters

Optionally, text is normalized (compatibility decomposition + canonical composition) so decorative or compatibility-form characters are spoken as their plain equivalents. A **supplementary mapping table** handles decorative letterforms that standard normalization leaves intact (e.g. enclosed/squared alphanumerics) by mapping them to their base letters, with deliberate exceptions for characters that are semantically meaningful as symbols. Boundary handling is surrogate-pair-aware throughout.

### 5.7 Markup generation for engines that consume it

Engines that accept marked-up input (the common case) require the sequence to be rendered into **balanced markup**. The methodology:

- A converter walks the Speech Sequence and maps commands to markup constructs: language-change → a language-scoped element; character-mode → an "interpret as characters" element; break → a timed pause element; pitch/rate/volume → prosody attributes (percentages relative to base); phoneme → a phoneme element with the IPA and fallback text; index → a named **mark** (this is what the engine echoes back as the feedback clock).
- A **balancer** maintains a stack of open elements. When an attribute (e.g. prosody) must change, it **closes and reopens** the affected elements to keep the markup well-formed, rather than emitting overlapping tags.
- Output is escaped, and code points illegal in the markup language are stripped or replaced.

For engines that take only plain text plus an out-of-band parameter API, the same sequence is instead consumed by setting engine parameters between text fragments and registering marks through the engine's bookmark/mark facility. The driver chooses; the sequence does not change. (See §6.)

---

## 6. The synthesizer-driver abstraction

The driver layer is what makes the engine replaceable. It defines a **contract**, a **settings model**, a **discovery/selection mechanism**, and — most importantly — the **feedback contract** that produces the master clock.

### 6.1 The driver contract

A driver must:

- **Declare capabilities**: the set of Speech-Sequence commands it honors, and the set of settings it exposes. Higher layers query these and adapt (§6.6).
- **Speak** a Speech Sequence: translate it to the engine's input (markup or text + parameter calls), inserting the engine's form of each index **mark**.
- **Cancel** immediately: stop current audio and abandon queued speech.
- **Pause/resume** if supported.
- **Clean up** on shutdown (and persist settings).

### 6.2 The settings model

Settings are described **declaratively** (name, type, range, label, whether per-voice). The standard set, normalized to a **0–100 scale** at the orchestration layer and mapped to each engine's native range inside the driver:

| Setting | Notes |
|---|---|
| Voice | The selected engine voice; many other settings are scoped per voice. |
| Variant | Voice variant/accent where the engine offers it. |
| Language | Speaking language (may be derived from available voices). |
| Rate | Speaking speed. |
| Rate-boost | A boolean that pushes speed *beyond* the engine's native maximum (§6.5). |
| Pitch | Base pitch. |
| Inflection | Pitch variability/expressiveness. |
| Volume | Output level. |

Normalizing to 0–100 means user settings and content-driven multipliers are engine-independent; the driver alone knows the engine's real parameter bounds.

### 6.3 Discovery, selection, and fallback

- Drivers are **auto-discovered** by scanning the drivers location and probing each with an availability check; unavailable engines are filtered out.
- A **fallback chain** guarantees the reader is never mute: prefer the configured engine; on failure fall through a priority list ending in a **null/“no speech” driver** that consumes sequences and tracks indices but produces no audio. The null driver is also the minimal reference implementation of the contract.
- Selecting an engine initializes its settings and fires a "engine changed" notification so dependent UI/state updates.

### 6.4 The feedback contract — how marks become the clock

This is the integration crux and the part most worth getting right:

1. The driver renders each **index mark** into the engine's native marker (a named mark in markup, or a bookmark via the engine's API).
2. As the engine synthesizes, it produces audio in segments and reports **mark events**. The driver must convert a mark event into an **"index reached" signal that fires when the corresponding audio has actually been *played*, not merely synthesized.**
   - With a pull/streaming engine that hands back audio plus mark positions, the driver tracks how many audio bytes correspond to each mark (mark audio-position × the engine's byte-rate **in the same time unit as the mark position** ⇒ byte offset) and attaches an **on-played callback** to that audio segment when it is queued to the audio output; the callback raises "index reached".
   - With an engine that plays audio itself and emits bookmark/mark callbacks, the driver maps those callbacks to the same "index reached" signal (ideally still gated on playback, e.g. via the engine's own playback events).
3. A separate **"done speaking"** signal fires when the engine finishes the whole request.
4. These signals are delivered back to the scheduler (marshalled onto the core thread), where they drive completion, callbacks, and cursor advancement (§3.2).

The key requirement for a faithful reimplementation: **"index reached" must track played audio.** If you fire it at synthesis time, continuous reading and cursor-following will run ahead of what the user hears.

> **Portability note.** A platform TTS *service/daemon* may already expose priority levels and index/mark callbacks. In that case you may not need to reimplement the byte-offset accounting yourself — you map the service's mark callbacks onto the scheduler's "index reached" signal. What you still must implement is everything in §3–§5: the priority/resume semantics, parameter replay, staleness cancellation, the fluency buffers, and the whole text-processing pipeline. A daemon does not do those for you.

### 6.5 Rate-boost — speed beyond the engine maximum

Engines cap their speaking rate. To exceed it (experienced users often read very fast), the driver uses one of two **engine-specific** strategies, exposed to the user as a single boolean:

- **Push the native rate parameter beyond its normal user range** — e.g. the reference formant engine multiplies its internal rate by a fixed factor (such as ×3); another engine simply exposes a higher maximum. No audio post-processing is involved.
- **Apply a pitch-preserving time-compression post-processor** to the engine's PCM output — used for engines whose native rate cannot be pushed further. A cross-platform, engine-agnostic time-scaling library speeds up the already-rendered audio (with a speed factor up to roughly ×6) while preserving pitch.

These are alternatives chosen per engine, **not** applied together; the driver hides whichever it uses behind the single boolean.

### 6.6 Capability negotiation and graceful degradation

Because drivers advertise capabilities, the layers above degrade gracefully instead of failing:

- If the engine does not support a **phoneme** command, the phoneme's plain-text fallback is spoken.
- If it does not support **character-mode**, spelling falls back to speaking characters individually with descriptions.
- If it cannot speak a requested **language/voice**, the condition is detected and optionally announced; the text is still spoken in the current voice.
- Unsupported prosody commands are simply omitted from what the driver emits.

This negotiation is why the same content logic drives a rich engine and a minimal one without special-casing.

---

## 7. The command and gesture (hotkey) system

The command system turns every user input — from a keyboard, a braille keyboard, a braille display's routing keys, or a touchscreen — into the invocation of a **script** (a named command). Its defining qualities are a **single unified input abstraction** and a **layered, overridable resolution order**.

### 7.1 The unified input gesture abstraction

Every input event, regardless of device, is wrapped as an abstract **gesture** object. Concrete kinds include keyboard gestures, braille-keyboard (dot) gestures, braille-display key/routing gestures, and touch gestures. Each gesture exposes a list of **identifiers** that name it. The rest of the system never special-cases the device — it only matches identifiers against a map. Adding a new input modality means adding a new gesture kind that produces identifiers; nothing else changes.

### 7.2 Gesture identifier format and namespacing

Identifiers are strings of the form:

```
source(variant):modifier+modifier+key
```

- **source** namespaces the device: keyboard, braille keyboard, a specific braille display model, touchscreen, etc.
- **variant** (optional, in parentheses) qualifies the source — e.g. the keyboard *layout profile* (desktop vs. laptop), or the touch *mode* (text/object).
- after the colon, an order-independent set of **modifiers** plus the terminal **key/action**.

Illustrative identifiers (device-neutral spelling):

| Identifier | Meaning |
|---|---|
| `keyboard(desktop):SR+upArrow` | keyboard, desktop layout, screen-reader-key + Up |
| `keyboard(laptop):SR+l` | keyboard, laptop layout, screen-reader-key + L |
| `keyboard:SR+space` | keyboard, **layout-agnostic** binding |
| `brailleKeyboard:space+dot1+dot3` | braille keyboard, space chord with dots 1 and 3 |
| `brailleDisplay(model):routing7` | a specific display's 7th routing key |
| `touch(object):2finger_flickRight` | touchscreen, object mode, two-finger right flick |
| `touch:tap` | touchscreen, **mode-agnostic** single tap |

**Normalization** makes matching robust: the identifier is lower-cased and the modifier set is **sorted**, so `SR+control+f1` and `control+SR+f1` are the same key. A single physical action typically yields **several** identifiers from most specific to most general (e.g. a desktop keypress yields both a `keyboard(desktop):…` and a layout-agnostic `keyboard:…` identifier); resolution tries them in that order.

### 7.3 The screen-reader modifier key

There is a dedicated **screen-reader modifier** (referred to in identifiers by a fixed token, written `SR` above; "NVDA key" in the reference). It is **not a fixed physical key** — it is a *configurable* role that the user assigns to one or more physical keys (conventionally a key such as Insert, or Caps Lock). This indirection is essential: it gives the screen reader a large private hotkey namespace that rarely collides with applications, and it lets users relocate it to a reachable key. **A reimplementation should treat the screen-reader modifier as a configurable abstraction from day one**, not hard-code a physical key.

### 7.4 Layout profiles

Two (or more) keyboard **layout profiles** coexist, chiefly:

- **Desktop** — assumes a numeric keypad; spatial review/navigation commands live on the numpad grid (see Appendix C).
- **Laptop** — assumes no numpad; the same commands are remapped onto letter/arrow combinations with the screen-reader modifier.

Both profiles' bindings are valid identifiers simultaneously; the active profile is tried first, then the layout-agnostic form. Designing for multiple profiles up front matters when targeting devices without numpads.

### 7.5 Gesture → script resolution order

When a gesture arrives, the system finds the script to run by walking an **ordered chain of objects** (most specific context to most global); at each object it consults the override maps scoped to that object, then that object's own scripts, stopping at the first match. Conceptually:

```
resolve(gesture):
    # walk the live object chain, most specific context → most global:
    for target in [ the gesture's own scriptable object,
                    active global plugins,
                    the focused application's module,
                    the active braille display (if scriptable),
                    active vision/enhancement providers,
                    the document interceptor (browse mode),   # honoring pass-through state
                    the focused object,
                    the focused object's ancestors,           # only scripts flagged "can propagate"
                    configuration-profile activation commands,
                    the global built-in command set ]:
        # for THIS object's class, the override maps are consulted first (in order),
        # then the object's own built-in / dynamically-assigned scripts:
        for id in gesture.identifiers (most specific → most general):
            if userMap   has (target.class, id):                return its script   # user overrides
            if localeMap has (target.class, id):                return its script   # locale defaults
            if activeBrailleDisplay.map has (target.class, id): return its script
        script = target.builtInOrDynamicScriptForAnyOf(gesture.identifiers)
        if script: return script

    return NONE   # → pass the input through to the OS/application
```

Two properties make this powerful:

1. **At each object, its override maps (user, then locale) are checked before that object's built-in scripts**, so any default can be rebound or unbound without touching code. Because the maps are scoped per object *class* and consulted as the chain is walked, an override only wins at its object's position — a built-in command on a more specific context (e.g. an application module) still outranks a rebind scoped to a more general one (e.g. the global command set).
2. **The object chain runs most-specific-context first** (application and document before global), so context-sensitive commands naturally shadow global ones, and a document in browse mode can claim keys (like single letters) that would otherwise type into a field.

### 7.6 Scripts and their metadata

A **script** is a named command annotated with metadata that the system relies on:

| Attribute | Purpose |
|---|---|
| **description** | Human-readable help text (translatable); spoken in input-help mode and shown in the rebinding UI. |
| **category** | Groups commands in the rebinding UI (e.g. Speech, Review, Mouse, Configuration). |
| **default gesture(s)** | Zero or more default bindings, often per layout profile. Overridable by user/locale maps. |
| **can-propagate** | Whether the script is eligible when its owning object is an *ancestor* of focus, not just focus itself. |
| **bypass-input-help** | Whether it still executes while input-help mode is active (used by the toggle that exits help). |
| **allow-in-sleep-mode** | Whether it runs while the screen reader is "asleep" for a given app. |
| **resume-continuous-read** | Whether running it resumes an interrupted say-all. |
| **speak-on-demand** | Whether its output is produced in the "on-demand" speech mode. |

Separating bindings from command identity (a binding is a string → (module, class, script) mapping) is what allows the same command to be reached from keyboard, braille, and touch simultaneously, and to be rebound freely.

### 7.7 The gesture-map data structure and cumulative layering

A gesture map is a dictionary: **normalized identifier → list of command references** (module + class + script name). Maps are **cumulative and layered** — built-in defaults (declared on the scripts), then locale overrides, then user overrides — with later layers adding to or overriding earlier ones. A special **"unbind"** entry (binding a gesture to "no script" for a class) lets a layer *remove* an inherited default. Maps are loaded from simple INI-style text, trivial to reproduce.

### 7.8 Input-help (learn) mode

A togglable **input-help mode** intercepts every gesture *before execution* and, instead of running the command, **speaks the gesture's name and the bound command's description**. This lets users explore the keyboard safely. Only commands flagged *bypass-input-help* (notably the toggle itself) still execute. This is a small but important learnability feature worth replicating.

### 7.9 Layered command targets (recap)

The same layering that governs gesture resolution governs *where commands live*: a global built-in set provides the baseline; application modules add or override per-app; the document interceptor adds browse-mode commands; the focused object can add object-specific commands. This mirrors the object-overlay and event-handler chains elsewhere in the system — one consistent "specific overrides general" principle.

### 7.10 Browse mode and single-letter quick navigation

Documents (web pages, rich documents) are presented through a **browse-mode interceptor** that exposes the content as a flat, navigable buffer. It runs in one of two sub-modes:

- **Browse (read) mode** — keys are commands. Arrows move through content; **single letters jump between elements of a type** (next heading, next link, next form field, etc.), and **shift+letter jumps backward**. Containers (lists, tables) can be entered/skipped as units.
- **Focus (forms) mode** — keys pass through to the control (so you can type). The system switches automatically when focus lands in an editable control, and a dedicated toggle forces the mode.

Quick-navigation is implemented as ordinary scripts on the interceptor, each parameterized by an element type, and a search over the content model in the requested direction. Because the interceptor sits *above* the focused object in the resolution chain (§7.5), it can legitimately claim single letters in read mode while releasing them in focus mode (pass-through). The full element-type table is Appendix B.

### 7.11 Multi-source unification — why it matters

Because all devices feed one map, a single command can carry bindings from every modality at once — e.g. continuous-read might be reachable by a keyboard chord, a braille display key, and a three-finger downward flick, all listed against the same command. A reimplementation should preserve this: define commands once, bind from many sources.

---

## 8. Implementation considerations for a different OS/language

This section translates the methodology into guidance for building an equivalent on another stack. The orchestration (§3–§4), text processing (§5), and command system (§7) are **platform-neutral and port directly as designs**. The platform-specific work is concentrated in two seams: **where the UI information comes from** and **where speech/braille/audio go out**.

### 8.1 The delegation boundary — what you build vs. what you delegate

| Concern | Build it (this document) | Delegate to the platform |
|---|---|---|
| What to say, when, with what prosody | ✔ Orchestration + text pipeline | |
| Command/gesture system | ✔ §7 | |
| Reading the UI (focus, tree, events, text) | adapter layer | platform **accessibility API** |
| Speech synthesis | driver contract (§6) | **TTS engine(s)** |
| Audio playout | the feedback wiring (§6.4) | platform **audio server** / TTS service |
| Braille translation | call the library | **braille translation library** |
| Braille device I/O | driver contract | **braille device service** |

### 8.2 Generic component map (with a concrete example stack)

Each platform-specific seam has a well-established counterpart. The middle column is the **generic role**; the right column shows one concrete, mature realization (a modern Linux desktop) as an existence proof.

| Platform seam (generic role) | What it does | Concrete example |
|---|---|---|
| **Accessibility API** | Exposes the app/UI accessibility tree, roles, states, text, and fires focus/caret/state events to assistive clients. | AT-SPI2 over the desktop bus |
| **TTS service / abstraction** | A daemon that accepts text + priority + marks and routes to engines; often already provides priority queues and mark callbacks. | speech-dispatcher (SSIP protocol) |
| **TTS engine(s)** | Actual synthesis; the replaceable backend behind the driver contract. | a formant engine and/or a neural engine |
| **Audio server** | Mixing/output; usually reached *through* the TTS service, not directly. | the system audio server |
| **Braille translation** | Text ↔ contracted/uncontracted braille via locale tables. | the standard open braille-translation library |
| **Braille device service** | Drives physical displays, exposes routing keys as input. | the braille device daemon |

A mature, same-shaped reference implementation exists in this space (a long-standing desktop screen reader built on exactly the accessibility-API + TTS-service + braille-library combination). **Study it as a structural template** for the platform seams, while using this document for the orchestration intelligence.

### 8.3 What a TTS service gives you — and what it does not

If your platform offers a TTS *service/daemon* (recommended over linking an engine directly — see §8.6), it likely already provides: engine selection, a priority model, and **index/mark callbacks**. In that case:

- **You can skip** the low-level audio plumbing and the byte-offset accounting of §6.4 — map the service's mark callbacks onto the scheduler's "index reached" signal instead.
- **You still must build** everything in §3–§5 and §7: the three-level priority/resume semantics (if the service's priorities don't match), parameter replay, staleness cancellation, say-all chaining, speak-without-pauses buffering, repetition suppression, the entire text-processing pipeline, and the command/gesture system. A TTS service speaks text; it has no idea what a screen reader *should* say or when.

Validate one assumption early: **does the service report marks as audio is *played* or as it is *synthesized*?** The whole feedback clock depends on the former (§6.4). If only synthesis-time marks are available, you must reintroduce a playback-tracking shim.

### 8.4 Concurrency model on another platform

Keep the **single-threaded core fed by a queue**. Asynchronous sources — accessibility events, engine/mark callbacks, input device events, timers — should all enqueue onto one core loop drained by a periodic pump. This preserves ordering and removes the need for locking in the hot path. The pump cadence should be short enough to feel instant but coarse enough to coalesce bursts.

### 8.5 Capturing input globally

A screen reader must observe input system-wide and often **consume** keys before applications see them. This is the single most platform-sensitive part of the command system:

- Some platforms let an assistive client grab global key events directly.
- Others (notably modern compositor-secured desktops) **forbid global key grabs** for security and require the compositor/OS to *mediate* which assistive client may receive keys. Expect to integrate with such a mediation mechanism rather than hooking input directly, and expect the available identifier to be a logical key symbol rather than a raw scan code.
- The unified gesture abstraction (§7.1–§7.2) insulates the rest of your system from these differences — only the gesture-producing layer changes per platform.

### 8.6 Licensing considerations

This is a real design constraint, not an afterthought — decide it **before** choosing how you reach the engine:

- **The reference screen reader is strong-copyleft.** Reusing its *source* obligates you to that license. This document conveys *methodology*, not code, specifically so a clean-room reimplementation can choose its own license.
- **Engine licenses vary and can be strong copyleft.** *Linking* a copyleft synthesis library into your process can pull your whole application under that license. The standard mitigation is to reach the engine **across a process boundary through a TTS service** whose *client library* is permissively/weak-copyleft licensed and which communicates over a socket/IPC — copyleft does not propagate across that boundary. This is an additional architectural reason to prefer a TTS service over direct engine linking.
- **Braille translation and accessibility client libraries** are typically weak-copyleft (linkable from differently-licensed code under their terms). Braille *tables* may carry their own assorted licenses.
- Net: a differently-licensed equivalent is feasible if you (a) clean-room the logic, (b) reach synthesis via a permissive service boundary rather than linking a copyleft engine, and (c) honor the weak-copyleft terms of the libraries you link.

### 8.7 Suggested build order

1. **Spine first:** core loop + queue + the Speech Sequence IR + a trivial "speak plain text via the TTS service" driver, with the index/mark feedback wired end-to-end. Prove the clock works.
2. **Scheduler:** priorities, interruption/resume, parameter replay, completion via marks.
3. **Input/command system:** the gesture abstraction, maps, resolution chain, a handful of commands. Now it's interactive.
4. **Reading from the platform:** the accessibility-API adapter producing objects, focus/caret events, and text — feed object presentation and repetition suppression.
5. **Text pipeline:** symbols, dictionaries, language tagging, normalization, markup.
6. **Fluency:** say-all chaining and speak-without-pauses.
7. **Browse mode:** the document interceptor and quick navigation.
8. **Braille** and **touch** as additional gesture sources and an additional output, reusing the same map and IR.

---

## Appendix A — Full command reference

> **Notation.** `SR` = the configurable screen-reader modifier key (conventionally Insert or Caps Lock). `numpad*` = numeric-keypad keys (desktop profile). `touch(...)` = touchscreen gesture. A dash (—) means no default binding (command exists but must be bound by the user). Spatial navigation commands (review cursor, object navigation) are in **Appendix C**; browse-mode quick-nav is in **Appendix B**. These are the reference implementation's defaults as extracted; treat them as a starting binding set to adapt, not gospel — verify against your own key-availability constraints.

### System & status
| Command | Desktop | Laptop |
|---|---|---|
| Report date and time | `SR+f12` | `SR+f12` |
| Read clipboard text | `SR+c` | `SR+c` |
| Report battery status | `SR+shift+b` | `SR+shift+b` |
| Show the screen-reader menu | `SR+n` | `SR+n` |
| Quit | `SR+q` | `SR+q` |
| Restart | — | — |
| Toggle screen curtain (blank screen) | `SR+control+escape` | `SR+control+escape` |
| Sleep mode for current app | `SR+shift+s` | `SR+shift+z` |

### System focus & window
| Command | Desktop | Laptop |
|---|---|---|
| Report current focus | `SR+tab` | `SR+tab` |
| Read all controls in active window | `SR+b` | `SR+b` |
| Report window title | `SR+t` | `SR+t` |
| Report status bar | `SR+end` | `SR+shift+end` |
| Report focused object's accelerator keys | `shift+numpad2` | `SR+control+shift+.` |
| Move to containing document | `SR+control+space` | `SR+control+space` |

### System caret (text cursor)
| Command | Desktop | Laptop |
|---|---|---|
| Read current line | `SR+upArrow` | `SR+l` |
| Read current selection | `SR+shift+upArrow` | `SR+shift+s` |
| Say all (caret → end) | `SR+downArrow` | `SR+a` |
| Report/show formatting at caret | `SR+f` | `SR+f` |
| Report details/summary | `SR+d` | `SR+d` |
| Report caret position dimensions | `SR+numpadDelete` | `SR+delete` |

### Speech control
| Command | Desktop | Laptop |
|---|---|---|
| Cycle speech mode | `SR+s` | `SR+s` |
| Toggle speak typed characters | `SR+2` | `SR+2` |
| Toggle speak typed words | `SR+3` | `SR+3` |
| Toggle speak command keys | `SR+4` | `SR+4` |
| Toggle report dynamic content changes | `SR+5` | `SR+5` |
| Cycle symbol (punctuation) level | `SR+p` | `SR+p` |
| Next / previous synth setting (the "ring") | `SR+control+rightArrow` / `+leftArrow` | `SR+shift+control+rightArrow` / `+leftArrow` |
| Increase / decrease current synth setting | `SR+control+upArrow` / `+downArrow` | `SR+shift+control+upArrow` / `+downArrow` |
| Increase / decrease (large step) | `SR+control+pageUp` / `+pageDown` | `SR+shift+control+pageUp` / `+pageDown` |
| Toggle progress-bar output | `SR+u` | `SR+u` |
| Repeat last spoken information | `SR+x` | `SR+x` |
| Toggle delayed character descriptions | — | — |
| Cycle automatic language switching | — | — |
| Cycle Unicode normalization (speech) | — | — |
| Toggle emoji/symbol-name (CLDR) reporting | — | — |

### Mouse
| Command | Desktop | Laptop |
|---|---|---|
| Left click | `numpadDivide` | `SR+[` |
| Right click | `numpadMultiply` | `SR+]` |
| Lock/unlock left button | `shift+numpadDivide` | `SR+control+[` |
| Lock/unlock right button | `shift+numpadMultiply` | `SR+control+]` |
| Move mouse to navigator object | `SR+numpadDivide` | `SR+shift+m` |
| Move navigator object to mouse | `SR+numpadMultiply` | `SR+shift+n` |
| Toggle mouse tracking | `SR+m` | `SR+m` |
| Scroll up/down/left/right; audio coords; text resolution | — | — |

### Configuration & dialogs
| Command | Binding |
|---|---|
| Save configuration | `SR+control+c` |
| Revert configuration | `SR+control+r` |
| General settings | `SR+control+g` |
| Synthesizer selection | `SR+control+s` |
| Speech (voice) settings | `SR+control+v` |
| Keyboard settings | `SR+control+k` |
| Mouse settings | `SR+control+m` |
| Braille display selection | `SR+control+a` |
| Document formatting | `SR+control+d` |
| Browse mode settings | `SR+control+b` |
| Object presentation | `SR+control+o` |
| Audio settings | `SR+control+u` |
| Configuration profiles | `SR+control+p` |
| Input gestures; symbol pronunciation; dictionaries; advanced; add-on store; review cursor; vision; etc. | — |

### Document-formatting reporting toggles
Each toggles whether the corresponding attribute is announced while reading. None bound by default. Coverage includes: font name, font size, font attributes (bold/italic/underline), super/subscript, revisions, emphasis, highlight, color, alignment, style, spelling errors (speech and braille), page, line number, line indentation, paragraph indentation, line spacing, tables, table headers, table cell coordinates, cell borders, links, link type, headings, lists, block quotes, articles, comments, graphics, frames, clickable, groupings, figures, landmarks.

### Audio
| Command | Binding |
|---|---|
| Cycle audio ducking mode | `SR+shift+d` |
| Cycle sound split (mono/stereo routing) | `SR+alt+s` |

### Braille (display control & toggles)
| Command | Binding |
|---|---|
| Toggle braille mode (follow-cursors ↔ speech-output) | `SR+alt+t` |
| Toggle tether (follow focus vs. review) | `SR+control+t` |
| Scroll back / forward | (display keys) |
| Move display to focus; previous/next line | (display keys) |
| Route to / activate cell; report formatting of cell | (routing keys) |
| Toggle auto-scroll; increase/decrease auto-scroll rate | — |
| Show/hide cursor; cycle cursor shape | — |
| Cycle show-messages; show-selection; routing-moves-caret | — |
| Toggle focus-context presentation; cycle Unicode normalization (braille) | — |
| Braille input: dots, enter, translate, erase last cell, select range, modifier chords | (braille keyboard) |

### Vision / magnification *(present in this reference build)*
| Command | Binding |
|---|---|
| Toggle magnifier | `SR+shift+w` |
| Start spotlight | `SR+shift+l` |
| Toggle color filter | `SR+shift+i` |
| Zoom in / out | `SR+shift+=` / `SR+shift+-` |
| Pan up/down/left/right | `SR+alt+arrow` |
| Pan to top/bottom/left/right edge | `SR+shift+alt+arrow` |
| Follow mouse / focus / navigator / review / all; fullscreen; move mouse to view | — |

### Tools & utilities
| Command | Binding |
|---|---|
| Show developer info for navigator object | `SR+f1` |
| Report app-module info | `SR+control+f1` |
| Recognize current screen with OCR | `SR+r` |
| Reload plugins | `SR+control+f3` |
| Speech viewer / braille viewer; open user config directory; add-on store | — |

### Miscellaneous
| Command | Binding |
|---|---|
| Toggle browse/focus (pass-through) mode | `SR+space` |
| Report link destination | `SR+k` |
| Report formatting | `SR+shift+f` |
| Interact with math content | `SR+alt+m` |
| Pass next key through to the OS | `SR+f2` |
| Toggle input-help (learn) mode | `SR+1` |
| Toggle desktop/laptop keyboard layout | — |

---

## Appendix B — Browse-mode single-letter quick navigation

In browse (read) mode, a single letter jumps to the **next** element of a type; **shift + letter** jumps to the **previous** one. Heading levels 1–9 use the digit keys (`shift+digit` for previous). Container entry/exit uses comma keys. Not every type is meaningful in every document.

| Key | Jumps to next… |
|---|---|
| `h` | heading |
| `1`–`9` | heading at level 1–9 |
| `t` | table |
| `k` | link |
| `u` | unvisited link |
| `v` | visited link |
| `b` | button |
| `f` | form field |
| `e` | edit (text) field |
| `c` | combo box |
| `x` | checkbox |
| `r` | radio button |
| `l` | list |
| `i` | list item |
| `g` | graphic / image |
| `q` | block quote |
| `n` | non-linked text |
| `s` | separator |
| `m` | frame |
| `d` | landmark |
| `o` | embedded object (audio/video/app/dialog) |
| `a` | annotation (comment/revision) |
| `p` | text paragraph |
| `w` | spelling error |

**Container navigation:** `shift+comma` → start of current container; `comma` → past end of current container.

---

## Appendix C — Spatial navigation command sets

These are the two independent spatial cursors and their movement grids. The **desktop** profile maps them onto the numeric keypad as a spatial mnemonic; the **laptop** profile remaps them onto modified arrows/letters for keypad-less devices. Many also carry touch flicks.

### The review cursor
A position **independent of the system caret**, used to explore content without moving focus. The desktop numpad forms a 3×3 grid: rows = line / word / character; columns = previous / current / next.

| Command | Desktop | Laptop | Touch |
|---|---|---|---|
| Previous / current / next **line** | `numpad7` / `numpad8` / `numpad9` | `SR+upArrow` / `SR+shift+.` / `SR+downArrow` | flick up / — / flick down |
| Previous / current / next **word** | `numpad4` / `numpad5` / `numpad6` | `SR+control+leftArrow` / `SR+control+.` / `SR+control+rightArrow` | 2-finger flick L / hover up / 2-finger flick R |
| Previous / current / next **character** | `numpad1` / `numpad2` / `numpad3` | `SR+leftArrow` / `SR+.` / `SR+rightArrow` | flick left / — / flick right |
| Start / end of line | `shift+numpad1` / `shift+numpad3` | `SR+home` / `SR+end` | — |
| Top / bottom of content | `shift+numpad7` / `shift+numpad9` | `SR+control+home` / `SR+control+end` | — |
| Previous / next page | `SR+pageUp` / `SR+pageDown` | `SR+shift+pageUp` / `SR+shift+pageDown` | — |
| Say all from review cursor | `numpadPlus` | `SR+shift+a` | 3-finger flick down |
| Cycle review mode (object/document/screen) prev/next | `SR+numpad1` / `SR+numpad7` | `SR+pageDown` / `SR+pageUp` | 2-finger flick down / up |
| Mark start; copy marked region | `SR+f9`; `SR+f10` | `SR+f9`; `SR+f10` | — |
| Toggle "caret moves review cursor" | `SR+6` | `SR+6` | — |

### Object navigation
Moves a **navigator object** through the accessibility hierarchy (parent / child / siblings), decoupled from focus.

| Command | Desktop | Laptop | Touch |
|---|---|---|---|
| Previous / next object (sibling) | `SR+numpad4` / `SR+numpad6` | `SR+shift+leftArrow` / `SR+shift+rightArrow` | 2-finger flick L / R |
| Parent / first child | `SR+numpad8` / `SR+numpad2` | `SR+shift+upArrow` / `SR+shift+downArrow` | flick up / down |
| Previous / next in flow | `SR+numpad9` / `SR+numpad3` | `SR+shift+[` / `SR+shift+]` | flick left / right |
| Report current navigator object | `SR+numpad5` | `SR+shift+o` | — |
| Navigator → focus | `SR+numpadMinus` | `SR+backspace` | — |
| Move focus → navigator | `SR+shift+numpadMinus` | `SR+shift+backspace` | — |
| Activate current object | `SR+numpadEnter` | `SR+enter` | double tap |
| Report navigator object dimensions | `SR+shift+numpadDelete` | `SR+shift+delete` | — |
| Toggle "focus moves navigator object" | `SR+7` | `SR+7` | — |

---

## Appendix D — Braille display gesture model

Braille displays contribute input through the **same** unified gesture system (§7.1). Two gesture sub-sources:

- **Braille keyboard (dot) chords** — combinations of the eight dot keys plus space (e.g. `space+dot1+dot3`). Used both for literal braille text entry (translated to characters) and for chorded commands. A generic "any dots" identifier supports the text-entry path; specific chords bind to commands.
- **Display hardware keys & routing keys** — namespaced by the **display model**, because button layouts differ per device. **Routing keys** (one per braille cell) are the signature interaction: pressing the routing key over a cell acts on the content shown there (move the caret/cursor to it, activate it, report its formatting, or extend a selection).

Each display driver may ship its **own default gesture map** binding that hardware to commands; users can rebind through the same map layering as keyboard. Core braille interactions exposed as commands include: scroll back/forward, move to previous/next line, route-to/activate cell, report-formatting of a cell, move display to focus, toggle tether (follow focus vs. review), select a cell range, and toggle speak-on-routing. The braille *output* path (cell rendering, contraction via the translation library, cursor display) is covered conceptually in §8.2; the point here is that braille **input** unifies with all other input.

---

## Appendix E — Touchscreen gesture model

Touch input is wrapped as gestures (§7.1) with a **mode** variant and a finger-count/action encoding, e.g. `touch(object):2finger_flickRight`, `touch(text):flickDown`, `touch:tap`. As with keyboard layouts, a **touch mode** (e.g. text vs. object) selects which set of bindings applies, and a mode-agnostic form is the fallback.

Representative built-in touch behaviors:

| Gesture | Behavior |
|---|---|
| `tap` / `hoverDown` | Report the object under the finger (explore by touch). |
| `hover` | Continuous explore as the finger moves. |
| `hoverUp` | Finger released. |
| `tapAndHold` | Right-click equivalent. |
| `double_tap` | Activate the current object. |
| `3finger_tap` | Cycle touch mode. |
| flicks (1/2/3-finger, directional) | Move by character/word/line, by object, change review mode, or start continuous read — mirroring the spatial sets in Appendix C. |

The same command is typically reachable by keyboard, braille, and touch at once (§7.11); touch simply adds another set of identifiers against the existing commands.

---

*End of reference. This document describes methodology and default bindings distilled from the NVDA screen reader for clean-room reimplementation; it intentionally contains no source code. Default bindings reflect the reference implementation as extracted and should be adapted to the target device's key availability and platform conventions.*
