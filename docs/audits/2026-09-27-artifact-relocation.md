# Local audit artifact boundary

The shared commit `763308b19566cad09184f884c9fda296843bdd89` included two local
audit workspaces with generated compiler outputs in the source index:

| Original directory under `docs/audits/` | Tracked files | Tracked blob bytes |
| --- | ---: | ---: |
| `lane-b-snapshot-20260927-010019` | 4,127 | 279,858,247 |
| `lane-d-scratch` | 8,548 | 1,395,399,999 |

`just ci` on that shared source reached `typos` and failed with exit 2 while
checking generated object-file names. Those artifacts also inflated every
source-custody scan. The language implementation was not implicated by this
failure.

The directories are preserved locally at `.audit/sep27-artifacts/` under the
same names and removed from the source index. No compiler outputs or audit
evidence are deleted from the working filesystem. The original committed
content remains recoverable from the Git commit above. Historical audit reports
retain their original snapshot paths; use this relocation record to find the
local copies. Durable lane reports remain tracked at their existing paths.

The existing `.audit/` policy is enforced by `source_snapshot()` before local
CI runs a group: force-adding any repository-ignored file now rejects instead
of silently treating generated state as source. A real-Git regression checks
root and nested ignore rules, preservation after untracking, independence from
personal excludes, and enumeration failure. Ordinary untracked ignored output
continues to be excluded from source custody.

This change does not rewrite history or claim to remove the blobs from the Git
object database. It restores the current source boundary and avoids reproducing
these local compiler artifacts in future source checkouts.
