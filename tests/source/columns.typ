
#table(
	columns: (1fr, 1fr),
	[abcedf], [b],
	[c], [d],
	[e],
)

#table(
	columns: (1fr, 1fr, 1fr),
  [a000000], [b],
	  [c0], [d00],
	[e], [],
)

#table(
	columns: (1fr, 1fr),
	[C], [D],
	[EEEEE], [FF],
)

#table(
	columns: (1fr, 1fr),
	table.header[A][B], [C], [D],
)

#table(
	columns: (1fr, 1fr),
	table.cell(rowspan: 2)[AC], [B], [D],
)




#table(
	columns: (1fr, 1fr, 1fr, 1fr),
	table.cell(colspan: 2)[AB], [C], [D],
	   [E], table.cell(colspan: 2)[FG] , [H],  
	   [I], [J] , table.cell(colspan: 2)[KL] ,
)


#table(
  columns: (1fr, 1fr, 1fr),
  table.cell(rowspan: 2)[AD], [B], [C],
  table.cell(rowspan: 2)[EH], [F],
  [G], table.cell(rowspan: 2)[HK],
  [I],[J],
)

#table(
  columns: (1fr, 1fr, auto),
  table.header([*Product*], [*Category*], [*Price*]),           [Apples],                                   [Produce],
  [\$1.23],                                                     [Oranges],                                  [Produce],
  [\$3.52],                                                     table.cell(colspan: 2)[*Produce Subtotal*], [*\$4.75*],
  [iPhone],                                                     [Electronics],                              [\$1000.00],
  table.footer(table.cell(colspan: 2)[*Total*], [*\$1004.75*]),
)
