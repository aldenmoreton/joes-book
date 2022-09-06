import React, {FC, useState, useMemo} from 'react'
import { useTable, Column } from 'react-table'
import columns from './TableColumns/databases.json'


type props = {
	data: readonly {}[]
}
export const PickTable: FC<props> = ({ data }) => {
	const COLUMNS = useMemo(() => columns, [])
	const DATA = useMemo(() => data, [])

	const tableInstance = useTable({
		columns: COLUMNS,
		data: DATA
	})

	const { getTableProps, getTableBodyProps, headerGroups, rows, prepareRow } = tableInstance

	return (
		<table {...getTableProps()}>
			<thead>
				{
					headerGroups.map(headerGroup => (
						<tr {...headerGroup.getHeaderGroupProps()}>
							{
								headerGroup.headers.map((column) => (
									<th {...column.getHeaderProps()}>{ column.render('Header') }</th>
								))
							}
						</tr>
					))
				}
			</thead>
			<tbody {...getTableBodyProps()}>
				{
					rows.map(row => {
						prepareRow(row)
						return (
							<tr {...row.getRowProps()}>
								{
									row.cells.map(cell => {
										return <td {...cell.getCellProps()}>{ cell.render('Cell') }</td>
									})
								}
							</tr>
						)
					})
				}
			</tbody>
		</table>
	)
}
