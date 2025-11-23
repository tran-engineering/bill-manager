#import "@preview/payqr-swiss:0.4.0": swiss-qr-bill

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
    account: "CH4431999123000889012",
    creditor-name: "tran-engineering - Khôi Tran",
    creditor-street: "Balmholzweg",
    creditor-building: "12",
    creditor-postal-code: "3145",
    creditor-city: "Niederscherli",
    creditor-country: "CH",
    amount: 1949.75,
    currency: "CHF",
    debtor-name: "Simon Muster",
    debtor-street: "Musterstrasse",
    debtor-building: "1",
    debtor-postal-code: "8000",
    debtor-city: "Seldwyla",
    debtor-country: "CH",
    reference-type: "SCOR",  // QRR, SCOR, or NON
    reference: "210000000003139471430009017",
    additional-info: "Bestellung vom 15.10.2020"
  )
]