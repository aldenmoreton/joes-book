import { InferGetServerSidePropsType } from 'next'
import BookCard from '../../components/BookCard'
import Typography from '@mui/material/Typography'
import SpeedDial from '@mui/material/SpeedDial'
import { SpeedDialIcon } from '@mui/material'
import { Button } from '@mui/material'
import SpeedDialAction from '@mui/material/SpeedDialAction'

export async function getServerSideProps() {
	return {
		props: {

		}
	}
}

export default function Home({ }: InferGetServerSidePropsType<typeof getServerSideProps>) {
	return (
		<>
			<Typography align='center' variant='h1'>Create new book</Typography>
		</>
	)
}