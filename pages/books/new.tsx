// import { InferGetServerSidePropsType } from 'next'

import Grid from "@mui/material/Grid";
import { Container } from "@mui/system";
import NewBook from "../../components/NewBook";
import PickSixGrid from "../../components/Picks/PickSix/PickSixGrid";


//{ }: InferGetServerSidePropsType<typeof getServerSideProps>
export default function Home() {
	return (
		<>
			<NewBook/>
		</>
	)
}