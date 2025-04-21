= Heading
== Table or Grid
#table(
    columns: (1fr, 1fr),
    align: (left, right),
    [a], [b],
    [longer], [],
    [c],
    [test],
)
== Equation
  $ x & = 13 y & \
  & & z? \
  2 y & = 13 * 2 y \
  \
  & \
$

Some math $c$, $d $, $ e$ and $ f_1^       pi * - 3$.
== Trailing comma and long blocks

// no trailing comma and content with only spaces
#figure(
    caption: [compact],
    [ Some Content ]
)

// no trailing comma and content with newline
#figure(
    caption: [nested],
    [
            Some Content ]
)

// trailing comma and content with newline
#figure(
    caption: [nested],
    [
            Some Content ],
)

== Linebreaks

Everything is written in content
mode is on the same line after formatting.
Maybe wanted or unwanted.
#image("image_a.png")
#image("image_b.png")

But paragraphs stay seperated!

== Label<section_label>

== Nesting

#let f = (diff, content ) => {

box(width: 30%,align(center,{
    content
    v(diff, weak:true)
    counter.display(numbering) + [ ] + caption
}))
}

== Code

#{
    let    x= 1 * - 3 ;
    let y=    1==2 and 3==3 
}