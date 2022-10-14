import Grid from "@mui/material/Grid";

import LooksOneIcon from '@mui/icons-material/LooksOne';
import LooksTwoIcon from '@mui/icons-material/LooksTwo';
import Looks3Icon from '@mui/icons-material/Looks3';
import Looks4Icon from '@mui/icons-material/Looks4';
import Looks5Icon from '@mui/icons-material/Looks5';
import Looks6Icon from '@mui/icons-material/Looks6';
import React from "react";
import ToggleButtonGroup from "@mui/material/ToggleButtonGroup";
import ToggleButton from "@mui/material/ToggleButton";
import Backdrop from "@mui/material/Backdrop";
import CircularProgress from "@mui/material/CircularProgress";
import Button from "@mui/material/Button";


function reverseSpread(spread: string) {
	return spread.includes('+') ? spread.replace('+', '-') : spread.replace('-', '+')
}

//TODO: point tracker
type props = {
	home: string,
	visitor: string,
	homeSpread: string,
	pointTracker: any
}
export default function PickSixForm( {home, visitor, homeSpread, pointTracker}: props ) {
	const [open, setOpen] = React.useState(false);
	const [points, setPoints] = React.useState<string | null>('undecided');
	const [pick, setPick] = React.useState<string | null>('undecided');

	const handleToggle = () => {
		setOpen(!open);
	  };
	  const handleClose = () => {
		setOpen(false);
	  };

	const handlePoints = (
	  event: React.MouseEvent<HTMLElement>,
	  newPoints: string | null,
	) => {
		if (newPoints && pointTracker[0][newPoints]) {
			return
		}
		let newPointTracker = pointTracker[0]
		if (newPoints) {
			newPointTracker[newPoints] = true
		}
		if (points) {
			newPointTracker[points] = false
		}
		pointTracker[1](newPointTracker)
		setPoints(newPoints? newPoints : 'undecided');
	};
	const handlePick = (
		event: React.MouseEvent<HTMLElement>,
		newPick: string | null,
	) => {
		setPick(newPick? newPick: 'undecided');
	  };
	console.log(points)
	return (
		<>
		<Backdrop
		sx={{ color: '#fff', zIndex: (theme) => theme.zIndex.drawer + 1 }}
		open={open}
		onClick={handleClose}
		>
		<CircularProgress color="primary" />
		</Backdrop>
		<form>
			<Grid container justifyContent="center">
				<ToggleButtonGroup
				value={points}
				color='primary'
				exclusive
				onChange={handlePoints}
				aria-label="points pick"
				>
					<ToggleButton value="one" aria-label="one" sx={{"&.MuiToggleButtonGroup-grouped": {border: "none"}}}>
						<LooksOneIcon fontSize="inherit" />
					</ToggleButton>
					<ToggleButton value="two" aria-label="two" sx={{"&.MuiToggleButtonGroup-grouped": {border: "none"}}}>
						<LooksTwoIcon fontSize="inherit" />
					</ToggleButton>
					<ToggleButton value="three" aria-label="three" sx={{"&.MuiToggleButtonGroup-grouped": {border: "none"}}}>
						<Looks3Icon fontSize="inherit" />
					</ToggleButton>
					<ToggleButton value="four" aria-label="four" sx={{"&.MuiToggleButtonGroup-grouped": {border: "none"}}}>
						<Looks4Icon fontSize="inherit" />
					</ToggleButton>
					<ToggleButton value="five" aria-label="five" sx={{"&.MuiToggleButtonGroup-grouped": {border: "none"}}}>
						<Looks5Icon fontSize="inherit" />
					</ToggleButton>
					<ToggleButton value="six" aria-label="six" sx={{"&.MuiToggleButtonGroup-grouped": {border: "none"}}}>
						<Looks6Icon fontSize="inherit" />
					</ToggleButton>
				</ToggleButtonGroup>
			</Grid>

			<Grid container justifyContent="center">
			<ToggleButtonGroup
				value={pick}
				color='primary'
				exclusive
				onChange={handlePick}
				aria-label="team pick"
				>
					<ToggleButton value="visitor" aria-label="visitor" sx={{"&.MuiToggleButtonGroup-grouped": {border: "none"}}}>
						<Button>{visitor + ' ' + reverseSpread(homeSpread)}</Button>
					</ToggleButton>
					<ToggleButton value="home" aria-label="home" sx={{"&.MuiToggleButtonGroup-grouped": {border: "none"}}}>
						<Button>{home + ' ' + homeSpread}</Button>
					</ToggleButton>
				</ToggleButtonGroup>
			</Grid>
		</form>
		</>
	)
}
