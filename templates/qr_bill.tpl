#import "@preview/payqr-swiss:0.4.0": swiss-qr-bill,

#set page(
  paper: "a4",
  margin: 0mm,
)
#place(
  bottom + left,
  dx: 0mm,
  dy: 0mm,
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