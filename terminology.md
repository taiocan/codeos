Ta celostni arhitekturni model in razvojni krog se v sodobnem AI-native inženirstvu imenuje **Declarative Behavioral Architecture (DBA)** ali bolj specifično **Intent-Driven Architecture (IDA)**, včasih opisan tudi kot **Continuous Behavioral Verification Loop**.

To je model, ki ga v svojem jedru implementira **Codeos**. Za razliko od tradicionalnih arhitektur (ki se osredotočajo na strukturo kode in podatkovne baze) se ta model osredotoča izključno na **tok in preverjanje namena (intent)** skozi celotno življenjsko dobo programske opreme.

Tukaj je razlaga, kako se ta specifični model imenuje v različnih inženirskih kontekstih in zakaj deluje kot sklenjena zanka:

---

### 1. Deklarativna vedenjska arhitektura (Declarative Behavioral Architecture)

Ime izhaja iz dejstva, da sistem deluje kot **reconciliation loop** (zanka usklajevanja), podobno kot Kubernetes, le da namesto infrastrukture usklajuje *obnašanje kode*.

* Vi podate **Intent** (kaj želite).
* Sistem preko **Behavior Contracts** in **Observed Runtime Graph** neprekinjeno primerja realno stanje z vašim nameravanim stanjem.

### 2. Arhitektura "Živega vira resnice" (Truth-First / Event-Centric Architecture)

Ker so v osrčju modela `Immutable Runtime Event Spine` in `Replayable System Truth`, ta model močno črpa iz naprednih konceptov **Event Sourcinga** in **CQRS**, vendar jih dviguje na nivo upravljanja celotnega življenjskega cikla aplikacije.

---

### Razčlenitev delovanja modela (The Engineering Loop)

Model je zasnovan kot navpični lijak, ki se na koncu spremeni v neskončno povratno zanko:

1. **Namenski del (Intent $\rightarrow$ Contracts):** Človek definira poslovno logiko (npr. preko BDD `Given-When-Then`). To ustvari strogo pogodbo obnašanja.
2. **Izvedbeni del (Modules $\rightarrow$ Event Spine):** AI implementira minimalno kodo, ki jo zaklene v dogodkovno hrbtenico. Vsaka akcija generira nespremenljiv dogodek (npr. tisti `correlation_id` in `state_value`, ki smo jih omenjali).
3. **Opazovalni del (Graph $\rightarrow$ Replayable Truth):** Sistem med delovanjem (runtime) riše dejansko sliko sistema. Ker so dogodki nespremenljivi, lahko celotno zgodovino delovanja kadarkoli "prevrtite nazaj" (*Replayable*) in natančno ugotovite, kje je prišlo do odstopanja.
4. **Prilagoditveni del (Refinement $\rightarrow$ Verification):** Če opazovano stanje (Reality) odstopa od pogodbe (Intent), sistem ne ugiba, ampak izvede ciljni popravek (*Targeted Refinement*) in zanka se znova zapre.

### Zakaj je ta model ključen za AI razvoj?

Večina današnjih AI orodij poskuša generirati kodo neposredno iz *Intent $\rightarrow$ Code*. To hitro vodi v sesutje sistema.

Ta model (Codeos model) pa vmes postavi **Behavior Contracts** (pogodbe), pod njih pa **Event Spine** (merljive dokaze). AI tako nima svobode, da bi ustvaril karkoli izven teh dveh mejnih zidov. Zato je končni rezultat sistema, zgrajenega po tem modelu, vedno predvidljiv in stabilen.







---

# Example System

We’ll use a realistic but simple system:

> A real-estate ingestion and searchable listing platform.

Goal:

* ingest listings from multiple agencies
* normalize them
* remove duplicates
* make them searchable
* continuously verify correctness

---

# 1. Intent

Human/business intent:

```text id="i1"
Continuously ingest real-estate listings from multiple sources
and provide fast, trustworthy searchable listings online.
```

This is NOT truth yet.
It is only desired outcome.

---

# 2. Behavior Contracts

Now we define observable truths.

These become authoritative.

---

## Contract A — Ingestion

```text id="bc1"
When a source sends a valid listing,
the system must emit ListingIngested.
```

---

## Contract B — Normalization

```text id="bc2"
Every ingested listing must produce exactly one normalized address.
```

---

## Contract C — Deduplication

```text id="bc3"
Two listings with same external_id + source
must resolve to one canonical listing.
```

---

## Contract D — Search Projection

```text id="bc4"
Every canonical listing must become searchable
within 5 seconds.
```

---

## Invariants

```text id="bc5"
listing_id is immutable

canonical_listing always references valid source listing

search index must never expose deleted listings
```

Now we have behavioral truth anchors.

---

# 3. Verified Behavioral Modules

Each module owns one behavioral responsibility.

---

# Module A — Feed Ingestion

## Input

```text id="m1"
Raw XML/JSON feed
```

## Output

```text id="m2"
ListingIngested
```

## Verification test

```text id="m3"
Given valid feed item
When parser runs
Then ListingIngested is emitted
```

---

# Module B — Address Normalization

## Input

```text id="m4"
ListingIngested
```

## Output

```text id="m5"
AddressNormalized
```

## Verification test

```text id="m6"
Given "5th Ave., NYC"
Then normalized form =
"5 Avenue, New York City"
```

---

# Module C — Deduplication

## Input

```text id="m7"
AddressNormalized
```

## Output

```text id="m8"
CanonicalListingCreated
```

## Verification test

```text id="m9"
Given same external_id twice
Then only one canonical listing exists
```

---

# Module D — Search Projection

## Input

```text id="m10"
CanonicalListingCreated
```

## Output

```text id="m11"
SearchProjectionUpdated
```

## Verification test

```text id="m12"
Given canonical listing
Then searchable within 5 seconds
```

---

# 4. Immutable Runtime Event Spine

Now runtime truth begins.

Every state transition becomes an immutable event.

---

# Actual runtime event flow

```text id="ev1"
RawFeedReceived
  ↓
ListingIngested
  ↓
AddressNormalized
  ↓
CanonicalListingCreated
  ↓
SearchProjectionUpdated
```

---

# Example event

```json id="ev2"
{
  "event_id": "evt-001",
  "event_type": "ListingIngested",
  "timestamp": 1710000000,
  "source_module": "feed_ingestion",
  "correlation_id": "listing-flow-777",
  "payload": {
    "listing_id": "abc123",
    "source": "agency_x"
  }
}
```

Important:

* immutable
* append-only
* replayable
* observable

This becomes operational truth.

---

# 5. Observed Runtime Graph

Now we stop looking at diagrams and observe REAL behavior.

The runtime graph emerges from actual execution.

---

# Observed graph

```text id="rg1"
FeedIngestion
   ↓
AddressNormalization
   ↓
Deduplication
   ↓
SearchProjection
```

But now imagine runtime observation reveals:

```text id="rg2"
FeedIngestion
   ↓
AddressNormalization
   ↓
SearchProjection
   ↓
Deduplication
```

That would expose a real architectural flaw:

* duplicates becoming searchable before resolution

This is the power of runtime-observed architecture.

---

# 6. Replayable System Truth

Now suppose:

* deduplication logic had a bug
* duplicates leaked into production

Traditional systems:

* patch DB manually
* inconsistent fixes
* uncertain truth

Your model:

* replay truth

---

# Replay process

```text id="rp1"
Historical RawFeedReceived events
   ↓
Replay through fixed DeduplicationModule
   ↓
Generate corrected canonical listings
   ↓
Rebuild search projection
```

No guessing.
Truth is reconstructible.

---

# Example replay guarantee

```text id="rp2"
Same events
+
Same module version
+
Same constraints

→ same resulting truth
```

That is deterministic operational verification.

---

# 7. Targeted Behavioral Refinement

Suppose runtime metrics reveal:

```text id="tr1"
duplicate_detection_accuracy = 82%
```

Observed failure:

* address abbreviations bypass deduplication

Instead of redesigning everything:

We refine ONLY the affected behavior.

---

# Refinement

Old contract:

```text id="tr2"
duplicate match = exact normalized address
```

New contract:

```text id="tr3"
duplicate match =
normalized address
+
geo proximity
+
fuzzy similarity
```

Then:

* update tests
* update module
* replay historical events

Minimal targeted evolution.

---

# 8. Continuous Verification

System never becomes “finished.”

Verification continuously runs against runtime reality.

---

# Continuous checks

## Behavioral verification

```text id="cv1"
All listings searchable < 5 seconds
```

---

## Invariant verification

```text id="cv2"
No deleted listing appears in search
```

---

## Runtime verification

```text id="cv3"
Every ListingIngested
must eventually produce either:
  CanonicalListingCreated
or
  ListingRejected
```

---

# Runtime observability metrics

```text id="cv4"
ingestion_rate/sec
deduplication_accuracy
projection_latency
replay_consistency
event_chain_integrity
```

System health becomes measurable truth.

---

# FULL END-TO-END FLOW

```text id="full1"
Intent
  "Create trustworthy searchable listings"
        ↓

Behavior Contracts
  "Listings must normalize, deduplicate, and index correctly"
        ↓

Verified Behavioral Modules
  ingestion
  normalization
  deduplication
  search projection
        ↓

Immutable Runtime Event Spine
  append-only operational truth
        ↓

Observed Runtime Graph
  actual runtime interaction flow
        ↓

Replayable System Truth
  deterministic reconstruction
        ↓

Targeted Behavioral Refinement
  improve only observed weak behaviors
        ↓

Continuous Verification
  invariants + metrics + runtime validation
```

---

# Final conceptual insight

Notice what happened:

The system is NOT fundamentally:

* database-centered
* API-centered
* class-centered
* service-centered

It is fundamentally:

```text id="insight1"
behavior-centered
+
runtime-truth-centered
```

That is the defining characteristic of your philosophy.

---

## Vocabulary-Centric Architecture

Terms for systems where types, statuses, or domain concepts are defined by a
configurable vocabulary and consumed across multiple modules.

See: `.codeos/patterns/vocabulary-architecture.md` for the full pattern reference.

| Term | Definition |
|---|---|
| **Concept** | The semantic identity the system reasons about, independent of any string representation. The unit of domain logic. |
| **Canonical** | The single stable runtime identifier chosen by the vocabulary to name a concept. Unique and authoritative — not merely "another representation." There is exactly one canonical per concept. |
| **Alias** | An alternative input form accepted for compatibility, migration, or user preference. Resolved to a concept before domain logic runs; never appears in business logic. |
| **Architecture Scope** | The single artifact for one project-level architectural scope. Its filename identifies the scope; it owns feature membership, structural decisions, and recorded human approval and cannot invent or alter behavior. |
| **Resolution** | Mapping any input form (alias or canonical string) to its concept via the vocabulary module's API. |
| **Vocabulary owner** | The module that defines concepts, accepts aliases, and exposes the resolution API. Exactly one owner per vocabulary. |
| **Vocabulary consumer** | Any module that operates on vocabulary-defined concepts by calling the resolution API. Consumers never inspect aliases or hardcode canonical strings. |
| **Concept Dependency Rule** | Business logic depends on vocabulary-defined concepts. Vocabulary resolution maps all representations to concepts before any domain comparison. Comparisons occur on resolved concept identity, not representations. |
| **Representation Ban Rule** | Domain layers must not store, compare, branch on, or pattern-match vocabulary representations (canonical strings or aliases). Only concept identity is valid in domain logic. |
| **Concept leak** | A bug where a vocabulary representation (alias string or assumed canonical form) escapes the resolution boundary and appears in domain logic, bypassing the vocabulary module. The R8 report_export bug is a concept leak. |
| **Normalize-on-write** | Resolution strategy: resolve aliases at ingestion; store concept identity. Simpler domain logic; vocabulary migrations require data backfill. |
| **Normalize-on-read** | Resolution strategy: store original representation; resolve at every comparison site. Flexible vocabulary evolution; each comparison site is a potential concept leak if resolution is accidentally skipped. |
