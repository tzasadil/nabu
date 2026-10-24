# Learned rules

Design mistakes and settled decisions mined from past OpenSpec sessions, each seen in more than one change. Design so they never reach review. The user request and a repo's `AGENTS.md` win over a rule: record the deviation in blueprint `## User Decisions` or `stack.md` decisions.

## Design

Target states that reviewers or the user had to enforce after the fact. Authors follow every rule. Each plan-time reviewer checks only its own heading.

### Opponent

Drop or simplify: machinery, layers, scope, unverified premises.

- Scope only what described flows reach. Limit, retry, recovery, or error case beyond the failure outcomes the Artifacts rules require -> needs an observed need, else backlog. One concern, one bound. (L1)
- A new component or layer needs a second consumer. With one, fold it into that consumer. (L2)
- A premise about existing behavior is checked in the source and cited by path before a decision rests on it. (L3)
- Logic, limits, and shapes that already have an owner in the repo are reused or extended, never copied. Search for the owner before adding one. (L10)
- A slice builds only what its own flows consume. Fields, routes, views, and schema for a later slice land in that slice. A field pulled forward needs a recorded reason that names the consuming slice. (L11)
- Each slice leaves the system working. No outage, dead control, or widened access stays open until a later slice: order the closing work first. (L23)
- A request finishes on the state it started with. Do not plan for changes that land while one request runs (access revoked, record edited, row deleted): the window is seconds, and the next request sees the new state. Queued and long-running work follows L24. (L28)
- Tests guard only what this change can break. No test of framework behavior, unreachable branches, or deleted code, and no second layer re-testing a rule another test owns. (L25)

### Artifacts

Add what is missing: tests, failure outcomes, recorded decisions.

- Each contract has one test task at the boundary that owns it, and that test can fail: no gate that passes on empty input or reuses the assumption it checks. (L4)
- Every data load names its failure outcome. A failed load never presents as empty or denied. (L5)
- Batch work states its per-item failure behavior. One failed item never aborts or blocks the rest. (L6)
- An `AGENTS.md` rule beats neighbouring code. A departure needs the user's approval, recorded in User Decisions. (L7)
- Client list contracts tolerate version skew. One invalid item never blanks the whole view or drops stored data. (L8)
- A change to an existing contract (removed or renamed field, route, or code, new failure mode) lists every consumer a repo-wide search finds: code, tests, fixtures, docs, and installed clients. Each gets a task or a recorded reason. A removal also names what it leaves unused and the behavior that stops. (L9)
- Every migration states its rollback intent. One that drops or rewrites live data also states its deploy order against the running code. (L12)
- A change that widens who can read data (public route, wider role, deleted content kept) records the exposure in User Decisions, approved by the user. (L13)
- Each requirement in the source (issue, stack, user request) maps to the design as written. Dropping, merging, adding, or reinterpreting one needs the user's approval, recorded in User Decisions. (L14)
- Every path and command in tasks resolves in the repo and runs as written from a named working directory. (L15)
- A final blueprint holds no undecided choice. Ask the user, then record the answer in User Decisions. Agent Decisions holds only choices the agent made alone. A premise still marked `UNCERTAIN` goes in Gotchas with the task that settles it. (L16)
- A new concern gets its own states, names, and copy. Reusing a state or term built for another concern inherits that concern's effects and meaning. (L20)
- Every deliberate skip, swallow, or fail-open path leaves an operator signal (log or audit). One that changes what the user sees also shows them a notice. (L21)
- Every state a flow enters on the way (running, syncing, pending, a held slot) names how it ends on failure, timeout, or crash. A permanent failure never loops on retry. The recovery mechanism may wait in backlog. (L22)
- A disable, revoke, delete, or narrowing edit states what queued or long-running work (jobs, syncs) does when it finishes. Stale work never overwrites it. Ordinary concurrent edits stay last-write-wins. (L24)


## Defaults

Answers the user gave the same way across changes. Apply them without asking. Ask only when the request contradicts one.

- A contract only the database enforces (constraint, trigger, single-statement atomicity) gets a real-database test through the repo's database test script. Everything else stays mocked. (L17)
