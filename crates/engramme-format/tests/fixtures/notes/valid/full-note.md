---
title: Downturn LGD
tags: [credit-risk, lgd]
sources:
  - Basel III, CRE36
aliases: [LGD]
---

# Downturn LGD

The downturn LGD reflects losses during an economic downturn, with
$\mathrm{LGD}_{dt} \geq \mathrm{LGD}_{lr}$ in general. #regulatory

$$
\mathrm{EL} = \mathrm{PD} \times \mathrm{LGD} \times \mathrm{EAD}
$$

:::qa {#c-91be0q}
What is the expected loss formula?

$$
\mathrm{EL} = ?
$$
---
$\mathrm{EL} = \mathrm{PD} \times \mathrm{LGD} \times \mathrm{EAD}$

```python
# not a separator nor a closer
---
:::
el = pd * lgd * ead
```
:::

:::qa
Which LGD is used for capital requirements?
---
The downturn LGD.
---
Not the long-run average.
:::

:::cloze {#c-7k2m9x}
$\mathrm{EL} = {{c1::\mathrm{PD}}} \times \mathrm{LGD} \times \mathrm{EAD}$,
where {{c1::PD}} is a probability and {{c2::$\frac{a}{b}$::a ratio}} is a ratio.
:::

:::cloze
```rust
let lgd = {{c1::1.0 - recovery_rate}};
```
:::
