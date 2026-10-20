# Block errors

:::note
Unknown type.
:::

::: qa
Space after the colons.
---
A
:::

:::qa junk
Q
---
A
:::

:::qa
A valid card between the errors.
---
Yes.
:::

:::qa {#c-12}
Q
---
A
:::

:::qa
Outer block, never closed.
:::cloze
Inner {{c1::block}}.
:::

:::qa
Last block, with a fence left open.
---
```
code
