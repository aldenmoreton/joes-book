import * as React from 'react';
import Card from '@mui/material/Card'
import CardHeader from '@mui/material/CardHeader';
import CardContent from '@mui/material/CardContent'
import Typography from '@mui/material/Typography'
import CardActions from '@mui/material/CardActions'
import Button from '@mui/material/Button'
import IconButton, { IconButtonProps } from '@mui/material/IconButton'
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import { styled } from '@mui/material/styles'
import Collapse from '@mui/material/Collapse'

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

export default function PickCard() {
	const [expanded, setExpanded] = React.useState(false);

  const handleExpandClick = () => {
    setExpanded(!expanded);
  };
	return (
		<Card raised sx={{ maxWidth:'350px' }}>
			<CardContent>
				<CardHeader
					title='Alabama at Texas'
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
			</CardContent>
			<CardActions style={{ justifyContent: 'center' }} disableSpacing>
				<Button>Alabama -7</Button>
				<Button>Texas +7</Button>
			</CardActions>
			<Collapse in={expanded} timeout="auto" unmountOnExit>
				<Typography variant='h5'>Broadcast Information:</Typography>
				<Typography variant='subtitle1'>September 10, 2:00PM CT</Typography>
				<Typography variant='subtitle1'>ESPN</Typography>
			</Collapse>
		</Card>
	)
}
