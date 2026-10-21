---
title: IFRS 9 staging
tags: [ifrs9, ecl]
sources:
  - IFRS 9 Financial Instruments, section 5.5
---

# IFRS 9 staging

IFRS 9 measures expected credit losses (ECL) in three stages: a 12-month ECL in
stage 1; a lifetime ECL in stage 2, after a significant increase in credit risk
since initial recognition, and in stage 3, once the exposure is
credit-impaired. The lifetime ECL sums discounted marginal losses:

$$
\mathrm{ECL} = \sum_{t=1}^{T} \mathrm{PD}_t \, \mathrm{LGD}_t \, \mathrm{EAD}_t \, D_t
$$

where $\mathrm{PD}_t$ is the marginal probability of default in period $t$ and
$D_t$ the discount factor at the effective interest rate.

:::cloze {#c-r9t2a6lx}
Stage 1 carries a {{c1::12-month}} ECL; stages 2 and 3 carry a {{c2::lifetime}} ECL.
:::

:::qa {#c-x5f0jh}
What moves an exposure from stage 1 to stage 2?
---
A significant increase in credit risk since initial recognition.
:::

:::cloze {#c-b8n3qy}
IFRS 9 rebuttably presumes a significant increase in credit risk once payments are more than {{c1::30}} days past due, and a default once they are {{c2::90}} days past due.
:::
