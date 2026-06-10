# Worker Start Here

1. Read `00-packet-scope-and-pinned-decisions.md` first. Cite decisions by
   `D#`; do not reinterpret them.
2. Lane map: 01→02→03 are one serial chain (worker A). 04 (worker B) and 05
   (worker C) fork after 03. 06 (worker D) lands its red rails first and is
   otherwise read-only. 07 is the only lane allowed on shared docs.
3. Hard rules inside every lane:
   - no git mutations (`checkout`/`restore`/`reset`/`stash`) — concurrent
     lanes have uncommitted work
   - `cargo fmt -p <crate>` only, never `--all`
   - fail-closed with a stable `composition_*` code; no heuristic fallback,
     no default match arms on new enums
   - discover exact output bytes by running the real binary before pinning
4. Definition of done for the packet is ticket 07's checklist, ending in
   `just ci`.
