import { InferGetServerSidePropsType } from 'next'

import { authOptions } from '../api/auth/[...nextauth]'
import { unstable_getServerSession } from "next-auth/next"
import { IncomingMessage, ServerResponse } from 'http'

interface Context {
	req: IncomingMessage & { cookies: Partial<{ [key: string]: string; }>; },
	res: ServerResponse,
	query: {
		id: string
	}
}
//TODO: Session type
export async function getServerSideProps(context: Context) {
	const session: any = await unstable_getServerSession(context.req, context.res, authOptions)

	const books = session.user.books.map((book: any) => {
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
			{/* <PickSixGrid cards={[{home: 'Texas', visitor: 'Alabama', homeSpread: '+7'}, {home: 'Geogia', visitor: 'Miss State', homeSpread: '-7'}]}/> */}
		</>
	)
}