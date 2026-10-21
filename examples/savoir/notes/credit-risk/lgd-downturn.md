---
title: Downturn LGD
tags: [credit-risk, lgd]
sources:
  - Basel Framework, CRE36
---

# Downturn LGD

The loss given default (LGD) is the share of the exposure at default that is
lost when the obligor defaults. A workout LGD discounts the recoveries $R_t$,
net of the costs $C_t$, received $t$ periods after default:

$$
\mathrm{LGD} = 1 - \frac{1}{\mathrm{EAD}} \sum_{t} \frac{R_t - C_t}{(1 + r)^{t}}
$$

where $r$ is the discount rate.

:::qa {#c-k3j9sd}
Why must the LGD used for IRB capital reflect an economic downturn?
---
Recoveries tend to fall when defaults cluster, so a long-run average LGD can understate the losses of a downturn; the downturn LGD is never below the long-run average.
:::

:::cloze {#c-p6e1rx}
In a workout LGD, recoveries and costs are discounted to the {{c1::date of default}}.
:::
