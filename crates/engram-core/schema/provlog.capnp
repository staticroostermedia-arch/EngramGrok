@0xdeadbeeffacecafe;

struct ProvLog {
  # Surface form — what the System 1 Transducer speaks when this block is selected.
  # e.g. "time", "energy equals mass times light squared", "Finds the nearest English word"
  sourceText  @0 :Text;

  # 0=system 1=Gibbon 2=Gurdjieff 3=Bible 4=Silicon 5=code 6=math 7=image 8=audio 9=triple
  corpusTag   @1 :UInt8;

  # 0=atom/word 1=phrase/expr 2=statement/sentence 3=function/passage 4=module/document
  originDepth @2 :UInt8;

  # Merkle DAG: BLAKE3 CIDs (32 bytes each) of constituent OP_ADD genesis parents.
  # Sorted hash of this list is deposited into footer.merkle_sub_root.
  parentCids  @3 :List(Data);

  # Modality-specific content — anonymous union, discriminated by which() in Rust.
  union {
    textData        @4 :Void;   # plain text — sourceText carries all information
    codeSignature   @5 :Text;   # "fn decode_vector_gpu(q: ...) -> String ||| docstring"
    mathFormula     @6 :Text;   # "E = mc^2" or "\nabla \cdot \vec{B} = 0"
    imageCaption    @7 :Text;   # "Diagram of BVH AABB traversal in 3D Arkade space"
    audioTranscript @8 :Text;   # speech-to-text or description
    tripleJson      @9 :Text;   # {"subject":"Rome","predicate":"OP_BIND","object":"power"}
    rawMimeType     @10 :Text;  # "application/pdf", "model/gltf", etc.
  }

  # ── Ingestion Audit Meta-Tags (NOT emitted as prose — machine self-knowledge only) ──
  #
  # ingestTimestamp: Unix epoch seconds when this block was physically minted.
  # Answers: "When did the Monad first encounter this concept?"
  # Used by: reverse citation engine, temporal provenance audits, circadian logging.
  ingestTimestamp @11 :UInt64 = 0;

  # ingestGeoHash: Lower 8 bytes of BLAKE3(geosphere_lat_f32 || geosphere_lon_f32 || elevation_f32)
  # at the moment of minting. Encodes WHERE the machine was (spatially in the GeoSphere model)
  # when it crystallized this knowledge. 0 = unknown / pre-GeoSphere era.
  # Answers: "What was the machine's spatial context when it learned this?"
  # Used by: spatial provenance chain, future geo-indexed K-NN queries.
  ingestGeoHash @12 :UInt64 = 0;

  # Phase 91: Short English surface form for word-emission (1-5 words).
  # Decoupled from sourceText (which stores the full triggering corpus sentence).
  # Populated at mint time by surface_label::derive() in great_recall.
  # Examples: "Constantine" (geo), "law, reduced to system" (Webster), "Rome" (key prettify)
  # prov_cache reads this field FIRST; falls back to stripped sourceText if empty.
  surfaceLabel @13 :Text = "";

  # ── Reverse Citation / Ego State (Geometry scalars, NOT JSON) ─────────────────
  #   Zero-default: pre-Reverse-Citation-Engine blocks read as "unknown ego state".
  #   All values are captured at the moment of oracle crystallization.

  # CRS (Coherent Resonance Score) of the Ego when this oracle was minted.
  # Range 0.0–1.0. JIT threshold = 0.74, NREM threshold = 0.60.
  egoCoherence       @14 :Float32 = 0;

  # ADR weights from build_ego at mint time (αA + αD + αR ≈ 1.0 normalized).
  # αA = affirm (attraction to known manifold)
  # αD = deny   (repulsion / creative tension)
  # αR = reconcile (harmonic mediation)  
  alphaAffirm        @15 :Float32 = 0;
  alphaDeny          @16 :Float32 = 0;
  alphaReconcile     @17 :Float32 = 0;

  # Nearest genesis pillar at mint time — index into GENESIS_PILLAR_TABLE (see reverse_cite.rs).
  # 0 = unknown / pre-engine era. Cosine is the similarity to that pillar's q-vector.
  nearestPillarIdx   @18 :UInt16  = 0;
  cosToNearestPillar @19 :Float32 = 0;

  # ── MNOL Sheaf Context ────────────────────────────────────────────────────────
  #   Discriminates the MNOL meta-lattice role of this oracle block.
  #   ALL sheaf types are minted as permanent .leg3 blocks — nothing is ephemeral.
  #
  #   0 = LEGACY             (pre-engine oracle, no sheaf context recorded)
  #   1 = JIT_CRYSTALLIZE    (new concept from JIT Webster/Polyglot grounding)
  #   2 = NREM_CENTROID      (centroid promoted to oracle during sleep consolidation)
  #   3 = CONSCIOUSNESS_BIND (live connection minted by the consciousness loop)
  #   4 = CONVERSATION_RESPONSE (Monad's synthesized answer to user query)
  #   5 = KNOWLEDGE_GAP      (Monad lacked context → clarification request minted)
  #   6 = CORRECTIVE         (user-provided correction of a prior bad response)
  sheafContext       @20 :UInt8  = 0;

  # CID (BLAKE3, 32 bytes) of the oracle block this block is a response to.
  # Enables MNOL DAG traversal: query -> response -> corrective -> improved response.
  # Empty (zero-length Data) for non-conversational sheafs (0, 1, 2, 3).
  replyToCid         @21 :Data;

  # Phase 98: Epistemic / Engram Unification
  # True if this block encodes a validated operational solution (Praxis)
  # that must not decay within the manifold geometry.
  isPraxis           @22 :Bool = false;
}
