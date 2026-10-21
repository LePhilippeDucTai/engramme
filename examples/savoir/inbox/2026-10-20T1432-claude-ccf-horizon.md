---
proposed_by: claude
date: 2026-10-20T14:32:00+02:00
context: Reviewing how the CCF of revolving credit lines is estimated
target: notes/credit-risk/ead-and-ccf.md
why: The reference date drives the realised CCF and is easy to get wrong
tags: [credit-risk, ead]
---

A realised CCF compares the amount drawn at default with the undrawn amount at
a reference date before default:

$$
\mathrm{CCF} = \frac{E_d - E_r}{L_r - E_r}
$$

where $E_d$ is the amount drawn at default, and $E_r$ and $L_r$ the amount
drawn and the limit at the reference date.

:::qa
At which date is the undrawn amount measured when estimating a realised CCF?
---
At a reference date before default, typically 12 months earlier, not at the date of default.
:::
