import * as React from 'react';
import Card from '@mui/material/Card'
import CardHeader from '@mui/material/CardHeader';
import CardContent from '@mui/material/CardContent'
import Typography from '@mui/material/Typography'
import CardActions from '@mui/material/CardActions'
import IconButton, { IconButtonProps } from '@mui/material/IconButton'
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import { styled } from '@mui/material/styles'
import Collapse from '@mui/material/Collapse'
import Container from '@mui/material/Container'
import PickSixForm from './PickSixForm'

interface ExpandMoreProps extends IconButtonProps {
	expand: boolean;
}

const ExpandMore = styled((props: ExpandMoreProps) => {
	const { expand, ...other } = props;
	return <IconButton {...other} />;
  })(({ theme, expand }) => ({
	transform: !expand ? 'rotate(0deg)' : 'rotate(180deg)',
	marginLeft: 'auto',
	transition: theme.transitions.create('transform', {
	  duration: theme.transitions.duration.shortest,
	}),
}));

type props = {
	home: string,
	visitor: string,
	homeSpread: string,
	pointTracker: [
		{ one: boolean; two: boolean; three: boolean; four: boolean; five: boolean; six: boolean; },
		React.Dispatch<React.SetStateAction<{ one: boolean; two: boolean; three: boolean; four: boolean; five: boolean; six: boolean; }>>
	]
}
export default function PickSixCard({home, visitor, homeSpread, pointTracker}: props) {
	const [expanded, setExpanded] = React.useState(false);

  const handleExpandClick = () => {
    setExpanded(!expanded);
  };
	return (
		<Card raised sx={{ maxWidth:'350px' }}>
			<CardContent>
				<CardHeader
					title={`${visitor} at ${home}`}
					// titleTypographyProps={{variant: 'subtitle1'}}
					subheader='SPRED'
					action={
						<ExpandMore
          					expand={expanded}
          					onClick={handleExpandClick}
          					aria-expanded={expanded}
          					aria-label="show more"
						>
          					<ExpandMoreIcon />
        				</ExpandMore>
					}
					/>
					<></>
			</CardContent>
			<CardActions style={{ justifyContent: 'center' }} disableSpacing>
				<PickSixForm home={home} visitor={visitor} homeSpread={homeSpread} pointTracker={pointTracker}/>
			</CardActions>
			<Collapse in={expanded} timeout="auto" unmountOnExit>
				<Container>
					<Typography variant='h5'>Broadcast Information:</Typography>
					<Typography variant='subtitle1'>September 10, 2:00PM CT</Typography>
					<Typography variant='subtitle1'>ESPN</Typography>
				</Container>
			</Collapse>
		</Card>
	)
}
