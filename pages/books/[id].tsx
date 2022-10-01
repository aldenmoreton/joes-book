import { InferGetServerSidePropsType } from 'next'

import { authOptions } from '../api/auth/[...nextauth]'
import { unstable_getServerSession } from "next-auth/next"

//TODO: Type definitions
export async function getServerSideProps(context: any) {
	const session: any = await unstable_getServerSession(context.req, context.res, authOptions)

	const books = session!.user!.books!.map((book: any) => {
		return book.toString()
	})

	const { id } = context.query
	if (!books.includes(id)) {
		return {
			redirect: {
				permanent: false,
				destination: "/"
			}
		}
	}

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