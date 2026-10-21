---
proposed_by: claude
date: 2026-10-20T14:32:00+02:00
context: Reviewing the downturn LGD estimation
target: notes/credit-risk/lgd-downturn.md
why: The downturn adjustment is a regulatory requirement that is easy to forget
tags: [credit-risk, lgd]
sources:
  - Basel III, CRE36
title: Kept in extra
---

The downturn LGD must exceed the long-run average, with
$\mathrm{LGD}_{dt} \geq \mathrm{LGD}_{lr}$.

:::qa
Why does capital use a downturn LGD rather than a long-run average?
---
Because losses given default rise in downturns, when defaults cluster.
:::

:::cloze
$\mathrm{EL} = {{c1::\mathrm{PD}::probability}} \times \mathrm{LGD} \times \mathrm{EAD}$
:::
