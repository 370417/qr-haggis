import * as React from 'react';
import * as ReactDOM from 'react-dom';

type Stage = "before game" | "play" | "wait" | "game over";

class App extends React.Component<{}, {}> {
    render() {
        return (
            <>
                <CardGrid stage="before game" />
                <Sidebar />
                <Scores />
            </>
        );
    }
}

type CardGridProps = {
    stage: Stage,
};

class CardGrid extends React.Component<CardGridProps> {
    render() {
        let rankLabels = [];
        let suitLabels = [];
        for (let rank = 2; rank <= 10; rank++) {
            rankLabels.push(<span>{rank}</span>);
        }
        for (let suit = 0; suit < 4; suit++) {
            suitLabels.push(<span>{suit}</span>);
        }
        let normalCards = [];
        let myWilcards = [];
        let opponentWildcards = [];
        for (let key = 0; key < 36; key++) {
            normalCards.push(<Card key={key} location={"unknown"} cardId={key} selected={false} stage={this.props.stage} />);
        }
        for (let key = 36; key < 39; key++) {
            myWilcards.push(<Card key={key} location={"unknown"} cardId={key} selected={false} stage={this.props.stage} />);
        }
        for (let key = 39; key < 42; key++) {
            opponentWildcards.push(<Card key={key} location={"unknown"} cardId={key} selected={false} stage={this.props.stage} />);
        }
        return <div id="card_grid">
            <div id="ranks"><span></span>{rankLabels}</div>
            <div id="suits">{suitLabels}</div>
            <div id="normal_cards">{normalCards}</div>
            <div id="my_wildcards">{myWilcards}</div>
            <div id="opponent_wildcards">{opponentWildcards}</div>
        </div>;
    }
}

type Location = "unknown" | "my_hand" | "table_just_played" | "table" | "captured_by_me" | "captured_by_opponent";

type CardProps = {
    cardId: number,
    location: Location,
    selected: boolean,
    stage: Stage,
};

class Card extends React.Component<CardProps> {
    render() {
        let className = "card";
        if (this.props.selected) {
            className += " selected";
        }
        className += ` ${this.props.location}`;
        return <div className={className}>{this.props.cardId}</div>;
    }
}

class Sidebar extends React.Component {
    render() {
        return (
            <>
                <QRDisplay />
                <Button />
                <QRReader />
            </>
        );
    }
}

class QRDisplay extends React.Component {
    render() {
        return <div></div>;
    }
}

class Button extends React.Component {
    render() {
        return <div></div>;
    }
}

class QRReader extends React.Component {
    render() {
        return <div></div>;
    }
}

class Scores extends React.Component {
    render() {
        return <div></div>;
    }
}

ReactDOM.render(<App />, document.body);
