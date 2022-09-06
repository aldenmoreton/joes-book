import { InferGetServerSidePropsType } from 'next'
import BookCard from '../../components/BookCard'
import Typography from '@mui/material/Typography'
import SpeedDial from '@mui/material/SpeedDial'
import { SpeedDialIcon } from '@mui/material'
import { Button } from '@mui/material'
import SpeedDialAction from '@mui/material/SpeedDialAction'
import { useRouter } from 'next/router'

export async function getServerSideProps(context: any) {
	const { id } = context.query
	console.log(id)
	return {
		props: {
			id: id
		}
	}
}

export default function Home({ id }: InferGetServerSidePropsType<typeof getServerSideProps>) {
	return (
		<>
			<p>You are on the book page</p>
			{id}
		</>
	)
}