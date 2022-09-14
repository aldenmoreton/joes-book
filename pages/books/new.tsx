// import { InferGetServerSidePropsType } from 'next'

import { Container } from "@mui/system";
import NewBook from "../../components/NewBook";


//{ }: InferGetServerSidePropsType<typeof getServerSideProps>
export default function Home() {
	return (
		<>
			<Container fixed>
				<NewBook/>
			</Container>
		</>
	)
}