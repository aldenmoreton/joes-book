import React from 'react';

import Grid from "@mui/material/Grid"
import PickSixCard from './PickSixCard';

type props = {
	cards: Array<PickSixCardInfo>
}
export default function PickSixGrid({cards}: props) {
	const defaultPoints = {
		one: false,
		two: false,
		three: false,
		four: false,
		five: false,
		six: false
	}

	return (
		<Grid container spacing={1} justifyContent='center'>
			{cards.map((card: PickSixCardInfo, idx: number) => {
				return (
					//TODO: Make cards fixed width
					<Grid item key={idx.toString()} sx={{maxWidth: 345, minWidth: 345}}>
						<PickSixCard home={card.home} visitor={card.visitor} homeSpread={card.homeSpread} pointTracker={React.useState(defaultPoints)}></PickSixCard>
					</Grid>
				)
			})}
		</Grid>
	)
}