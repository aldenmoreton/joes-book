import { InferGetServerSidePropsType } from 'next'

import BookGrid from '../../components/BookGrid'
import Typography from '@mui/material/Typography'
import { SpeedDialIcon } from '@mui/material'
import { Button } from '@mui/material'

import { authOptions } from '../api/auth/[...nextauth]'
import { unstable_getServerSession } from "next-auth/next"

import clientPromise from "../../lib/mongodb"
import { getBookCardProps } from '../../lib/books'

//TODO: context interface
//TODO: Type definitions
export async function getServerSideProps(context: any) {
	const session: any = await unstable_getServerSession(context.req, context.res, authOptions)
	const client = await clientPromise
	const bookIds = session.user.books

	if (bookIds.length > 0) {
		const cards = await getBookCardProps(client, bookIds)

		return {
			props: {
				cards: cards
			}
		}
	}

	return {
		props: {
			cards: null
		}
	}

}

export default function Home({ cards }: InferGetServerSidePropsType<typeof getServerSideProps>) {
	if (!cards) {
		return (
			<>
				<h1>You don't have any books</h1>
			</>
		)
	}

	return (
		<>
			<Typography align='center' variant='h4'>Your current books!</Typography>
			<BookGrid cards={cards}></BookGrid>
			<Button variant='contained' sx={{ position: 'fixed', bottom: 16, right: 16, borderRadius:10}} href='books/new'>
				<SpeedDialIcon></SpeedDialIcon>
			</Button>
		</>
	)
}