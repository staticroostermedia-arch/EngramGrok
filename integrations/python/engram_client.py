"""
engram_client.py — Drop-in Python client for the Engram REST memory server.

Copy this file into any project. No dependencies beyond `requests`.

Usage:
    from engram_client import EngramClient

    mem = EngramClient()                     # default: http://localhost:3456
    mem.remember("my_bug_fix", "Fixed X by doing Y")
    results = mem.recall("bug fixes", k=5)
    for r in results:
        print(r)

Start the server first:
    engram serve --port 3456 --store ~/.engram/manifold

Docs: https://github.com/staticroostermedia-arch/engram/blob/master/docs/rest_api.md
"""

from __future__ import annotations

import os
from dataclasses import dataclass, field
from typing import Optional

try:
    import requests
except ImportError:
    raise ImportError(
        "engram_client requires 'requests'. Install it with: pip install requests"
    )


# ── Data models ──────────────────────────────────────────────────────────────

@dataclass
class Memory:
    """A single memory block returned by recall() or trace()."""
    concept: str
    score: float
    crs: float
    text: str
    explain: Optional[str] = None

    def __repr__(self) -> str:
        crs_label = (
            "PINNED" if self.crs >= 1.0
            else "GROUNDED" if self.crs >= 0.74
            else "HYPOTHESIS" if self.crs >= 0.50
            else "UNCERTAIN"
        )
        return (
            f"Memory(concept={self.concept!r}, score={self.score:.3f}, "
            f"crs={self.crs:.3f} [{crs_label}])"
        )


@dataclass
class RecentEntry:
    """A recently accessed concept entry."""
    concept: str
    last_accessed: int  # Unix timestamp
    ago: str            # Human-readable (e.g. "2m ago")


# ── Client ───────────────────────────────────────────────────────────────────

class EngramClient:
    """
    Thin Python wrapper around the Engram REST API.

    Parameters
    ----------
    base_url : str
        Base URL of the Engram server. Default: http://localhost:3456
    api_key : str, optional
        Bearer token if ENGRAM_API_KEY is set on the server.
        Reads from the ENGRAM_API_KEY environment variable if not provided.
    timeout : float
        Request timeout in seconds. Default: 10.0
    """

    def __init__(
        self,
        base_url: str = "http://localhost:3456",
        api_key: Optional[str] = None,
        timeout: float = 10.0,
    ) -> None:
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout
        self._session = requests.Session()

        key = api_key or os.environ.get("ENGRAM_API_KEY")
        if key:
            self._session.headers["Authorization"] = f"Bearer {key}"

        self._session.headers["Content-Type"] = "application/json"

    # ── Internal ─────────────────────────────────────────────────────────────

    def _post(self, path: str, payload: dict) -> dict | list:
        url = f"{self.base_url}{path}"
        resp = self._session.post(url, json=payload, timeout=self.timeout)
        resp.raise_for_status()
        return resp.json()

    def _get(self, path: str, params: Optional[dict] = None) -> dict | list:
        url = f"{self.base_url}{path}"
        resp = self._session.get(url, params=params, timeout=self.timeout)
        resp.raise_for_status()
        return resp.json()

    # ── Health ────────────────────────────────────────────────────────────────

    def health(self) -> dict:
        """Check if the server is running. Returns {'status': 'ok', 'version': '...'}.

        Raises requests.ConnectionError if the server is unreachable.
        """
        return self._get("/health")

    def is_alive(self) -> bool:
        """Returns True if the server is reachable, False otherwise."""
        try:
            return self.health().get("status") == "ok"
        except Exception:
            return False

    # ── Core memory operations ────────────────────────────────────────────────

    def remember(self, concept: str, text: str) -> str:
        """
        Encode text and store it as a persistent memory block.

        Parameters
        ----------
        concept : str
            Unique snake_case key (e.g. 'auth_bug_2026_04'). Namespacing
            with '__' is recommended for related concepts.
        text : str
            The content to encode. PII (SSN, credit cards, emails) is
            automatically scrubbed server-side before storage.

        Returns
        -------
        str
            The server's status message.
        """
        result = self._post("/api/remember", {"concept": concept, "text": text})
        return result.get("message", "")

    def recall(
        self,
        query: str,
        k: int = 5,
        explain: bool = False,
        min_crs: float = 0.0,
    ) -> list[Memory]:
        """
        Semantic search. Returns the top-k memories most similar to the query.

        Parameters
        ----------
        query : str
            Natural language query (e.g. 'authentication bugs').
        k : int
            Number of results to return (1–20). Default: 5.
        explain : bool
            If True, each result includes a human-readable score explanation.
        min_crs : float
            Filter results by minimum CRS score. Default: 0.0 (no filter).
            Use 0.74 to get only grounded facts.

        Returns
        -------
        list[Memory]
            Results sorted by descending semantic similarity score.
        """
        raw = self._post("/api/recall", {"query": query, "k": k, "explain": explain})
        memories = [
            Memory(
                concept=m["concept"],
                score=m["score"],
                crs=m["crs"],
                text=m["text"],
                explain=m.get("explain"),
            )
            for m in raw
        ]
        if min_crs > 0.0:
            memories = [m for m in memories if m.crs >= min_crs]
        return memories

    def forget(self, concept: str) -> str:
        """
        Permanently delete a memory block.

        Warning: This destroys the block's entire history. To update a
        memory's content, call remember() again with the same concept name.

        Parameters
        ----------
        concept : str
            The concept name to delete.

        Returns
        -------
        str
            The server's status message.
        """
        result = self._post("/api/forget", {"concept": concept})
        return result.get("message", "")

    def relate(self, concept_a: str, concept_b: str, label: str) -> str:
        """
        Create a directional knowledge graph edge between two concepts.

        Parameters
        ----------
        concept_a : str
            Source concept (must already exist in the manifold).
        concept_b : str
            Target concept (must already exist in the manifold).
        label : str
            Relation type (e.g. 'depends_on', 'implements', 'contradicts',
            'fixes', 'derived_from').

        Returns
        -------
        str
            The server's status message.
        """
        result = self._post("/api/relate", {
            "concept_a": concept_a,
            "concept_b": concept_b,
            "label": label,
        })
        return result.get("message", "")

    def trace(
        self,
        term_a: str,
        term_b: str,
        op: str = "ADD",
        k: int = 5,
    ) -> list[Memory]:
        """
        VSA geometry query. Finds memories near the vector result of an
        operation on two concepts.

        Operations
        ----------
        - **ADD** (superposition): memories in the *union* of both concepts'
          semantic space. Use to broaden a search across two topics.
        - **BIND** (association): memories encoding the *relationship between*
          two concepts. Use to find how two ideas connect.

        Parameters
        ----------
        term_a : str
            First concept name or raw text query.
        term_b : str
            Second concept name or raw text query.
        op : str
            'ADD' or 'BIND'. Default: 'ADD'.
        k : int
            Number of results (1–20). Default: 5.

        Returns
        -------
        list[Memory]
            Results sorted by descending similarity to the computed vector.

        Example
        -------
        # What exists at the intersection of "auth" and "performance"?
        results = mem.trace("authentication", "performance", op="ADD")
        """
        raw = self._post("/api/trace", {
            "term_a": term_a,
            "term_b": term_b,
            "op": op.upper(),
            "k": k,
        })
        return [
            Memory(
                concept=m["concept"],
                score=m["score"],
                crs=m["crs"],
                text=m["text"],
                explain=m.get("explain"),
            )
            for m in raw
        ]

    # ── Listing & navigation ──────────────────────────────────────────────────

    def list_concepts(self) -> list[str]:
        """Return all stored concept names."""
        return self._get("/api/list")

    def recent(self, n: int = 10) -> list[RecentEntry]:
        """
        Return the N most recently accessed concepts.

        Parameters
        ----------
        n : int
            Number of entries to return (max 100). Default: 10.

        Returns
        -------
        list[RecentEntry]
            Sorted by most-recently-accessed first.
        """
        raw = self._get("/api/recent", params={"n": n})
        return [
            RecentEntry(
                concept=e["concept"],
                last_accessed=e["last_accessed"],
                ago=e["ago"],
            )
            for e in raw
        ]

    # ── Convenience helpers ───────────────────────────────────────────────────

    def remember_many(self, items: dict[str, str]) -> dict[str, str]:
        """
        Store multiple memories in one call.

        Parameters
        ----------
        items : dict[str, str]
            Mapping of {concept: text}.

        Returns
        -------
        dict[str, str]
            Mapping of {concept: status_message}.
        """
        return {concept: self.remember(concept, text) for concept, text in items.items()}

    def recall_grounded(self, query: str, k: int = 5) -> list[Memory]:
        """Recall only grounded facts (CRS >= 0.74)."""
        return self.recall(query, k=k, min_crs=0.74)


# ── CLI smoke test ────────────────────────────────────────────────────────────

if __name__ == "__main__":
    import sys

    url = sys.argv[1] if len(sys.argv) > 1 else "http://localhost:3456"
    client = EngramClient(base_url=url)

    print(f"Connecting to Engram at {url}...")
    if not client.is_alive():
        print("❌ Server not reachable. Start it with: engram serve --port 3456")
        sys.exit(1)

    health = client.health()
    print(f"✓ Engram {health['version']} is running\n")

    # Round-trip test
    print("Writing test memory...")
    client.remember("engram_py_test", "Python client smoke test — delete me.")

    print("Recalling...")
    results = client.recall("python smoke test", k=3)
    for r in results:
        print(f"  {r}")

    print("\nRecent concepts:")
    for entry in client.recent(n=5):
        print(f"  {entry.concept} ({entry.ago})")

    print("\nCleaning up test memory...")
    client.forget("engram_py_test")
    print("✓ All tests passed.")
