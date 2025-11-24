#import "@preview/payqr-swiss:0.4.0": swiss-qr-bill

#set page(
  paper: "a4",
  margin: 10mm,
)

#set text(font: "Roboto")

#align(right)[
  #image(
    "logo.svg",
    width: 45mm,
  )
]
#set text(size: 9pt)
*Rechnungssteller*

{{creditor-name}} \
{{creditor-street}} {{creditor-building}} \
{{creditor-postal-code}} {{creditor-city}} \
{{creditor-country}}

#columns(2)[
  #set text(size: 9pt)
  *Kunde*

  {{client-name}} \
  {{client-street}} {{client-building}} \
  {{client-postal-code}} {{client-city}} \
  {{client-country}}


  #colbreak()
  *Rechnungsadresse*

  {{debtor-name}} \
  {{debtor-street}} {{debtor-building}} \
  {{debtor-postal-code}} {{debtor-city}} \
  {{debtor-country}}
]

#box(width: 90%, inset: (top: 2em))[
  = Leistungen

  #table(
    inset: 1em,
    columns: (1fr, auto, auto, auto, auto),
    align: (x, y) => if x < 2 { left } else { right },
    stroke: (x, y) => if y == 0 {
      (bottom: 1pt + black)
    } else if y > 0 and {{table-rows}} > 0 and y == {{table-rows}} {
      (top: 0.5pt + black, bottom: 2pt + black)
    } else {
      (bottom: 0.2pt + black)
    },
    table.header([*Beschreibung*], [*Typ*], [*Anzahl*], [*Preis*], [*Total*]),
    {{table-contents}}
  )
]

#place(
  bottom + left,
  dx: -10mm,
  dy: 10mm,
)[
  #swiss-qr-bill(
    account: "{{account}}",
    creditor-name: "{{creditor-name}}",
    creditor-street: "{{creditor-street}}",
    creditor-building: "{{creditor-building}}",
    creditor-postal-code: "{{creditor-postal-code}}",
    creditor-city: "{{creditor-city}}",
    creditor-country: "{{creditor-country}}",
    amount: {{amount}},
    currency: "{{currency}}",
    debtor-name: "{{debtor-name}}",
    debtor-street: "{{debtor-street}}",
    debtor-building: "{{debtor-building}}",
    debtor-postal-code: "{{debtor-postal-code}}",
    debtor-city: "{{debtor-city}}",
    debtor-country: "{{debtor-country}}",
    reference-type: "{{reference-type}}",
    reference: "{{reference}}",
    additional-info: "{{additional-info}}",
  )
]
