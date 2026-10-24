# Rule writing

Copied from the `writing-skills` skill, trimmed to what shapes rule text. Rule lines are skill text: the style below applies to each one.

## Style

- Compact simple english, snappy bullets. Fragments OK.
- Arrows: causality `=>`, flow `->`.
- Model generalizes => give the idea or intent, skip scenario catalogs. Positive intent is enough. Negative rules and examples IFF observed, never prematurely.
- Wordy tells -> compress:
  - `such as A, B, or C` lists -> parenthesized examples or none.
  - Why as a `, so ...` clause or `Why:` prefix -> `X => Y`.
  - Doubled emphasis (`only ... nowhere else`) -> say it once.
  - Reader-addressing (`so the reader knows ...`) -> drop.

## Steering

- Outcomes and decision boundaries > itineraries.
- **No speculative guardrails.** Target rule alone by default. Guardrail IFF bad behavior observed in a real run. Imagined-failure guardrails add noise and can worsen behavior.
- State the target positively. Earned guardrail: pair it with the target.
- Steering away from a default: name the concrete pattern. Vague ban ("avoid generic") swaps one default for another.
- Leading words: reuse a short familiar anchor (`tracer bullet`, `red`) instead of redefining it. Same term everywhere. Coined term -> define it.

## Pruning

- **Single source:** one authoritative location per rule.
- **Relevance:** every line changes behavior. Drop exposition and dead branches.
- **Sediment and tombstones:** delete stale layers. The file describes the present: no "X is no longer needed" note.
- **No-ops:** delete rules that match the model default ("think carefully").

## Failure modes

- **Sprinkling:** one user ask = one rule, at the step it lives. No echo through phases, prompts, headings, recaps. Emphasis for real traps only.
- **Restated globals:** a rule copying user rules or `AGENTS.md` adds tokens and drifts. Check the globals first. Keep only the delta. Review lanes list every rule they check => duplication ok there.
