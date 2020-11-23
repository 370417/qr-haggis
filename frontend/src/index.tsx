import * as React from 'react';
import * as ReactDOM from 'react-dom';

type Stage = "before game" | "play" | "wait" | "game over";

type AppState = {
    stage: Stage,
    my_score: number,
    opponent_score: number,
    qrUrl: string,
};

class App extends React.Component<{}, AppState> {
    constructor() {
        super({});
        this.state = {
            stage: "before game",
            my_score: 0,
            opponent_score: 0,
            qrUrl: "",
        };
    }

    render() {
        return (
            <div id="app">
                <CardGrid stage={this.state.stage} />
                <Sidebar stage={this.state.stage} qrUrl={this.state.qrUrl} my_score={this.state.my_score} opponent_score={this.state.opponent_score} />
                <Scores my_score={this.state.my_score} opponent_score={this.state.opponent_score} />
            </div>
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
        let wildcardLabels = ['J', 'Q', 'K'];
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
            <div id="my_wildcards">{myWilcards}{wildcardLabels.map(label => <span>{label}</span>)}</div>
            <div id="opponent_wildcards">{opponentWildcards}{wildcardLabels.map(label => <span>{label}</span>)}</div>
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

type SidebarProps = {
    stage: Stage,
    my_score: number,
    opponent_score: number,
    qrUrl: string,
};

class Sidebar extends React.Component<SidebarProps> {
    render() {
        switch (this.props.stage) {
            case "before game":
                return <>
                    <Button stage={this.props.stage} my_score={this.props.my_score} opponent_score={this.props.opponent_score} />
                    <QRReader />
                </>;
            case "play":
                return <Button stage={this.props.stage} my_score={this.props.my_score} opponent_score={this.props.opponent_score} />
            case "wait":
                return <>
                    <QRDisplay qrUrl={this.props.qrUrl} />
                    <Button stage={this.props.stage} my_score={this.props.my_score} opponent_score={this.props.opponent_score} />
                    <QRReader />
                </>;
            case "game over":
                return <>
                    <QRDisplay qrUrl={this.props.qrUrl} />
                    <Button stage={this.props.stage} my_score={this.props.my_score} opponent_score={this.props.opponent_score} />
                </>;
        }
    }
}

type QRDisplayProps = {
    qrUrl: string,
};

class QRDisplay extends React.Component<QRDisplayProps> {
    render() {
        return <img id="qr_display" src={this.props.qrUrl} />;
    }
}

type ButtonProps = {
    stage: Stage,
    my_score: number,
    opponent_score: number,
};

class Button extends React.Component<ButtonProps> {
    render() {
        switch (this.props.stage) {
            case 'before game':
                return <div id="button">start</div>;
            case 'play':
                return <div id="button">play</div>;
            case 'wait':
                return <div id="button">wait</div>;
            case 'game over':
                if (this.props.my_score > this.props.opponent_score) {
                    return <div id="button">you won</div>;
                } else if (this.props.my_score < this.props.opponent_score) {
                    return <div id="button">you lost</div>;
                } else {
                    return <div id="button">you tied</div>;
                }
        }
    }
}

class QRReader extends React.Component {
    render() {
        return <div id="qr_reader"></div>;
    }
}

type ScoresProps = {
    my_score: number,
    opponent_score: number,
};

class Scores extends React.Component<ScoresProps> {
    render() {
        return <div id="scores">{this.props.my_score}pts / {this.props.opponent_score}pts</div>;
    }
}

ReactDOM.render(<App />, document.body);
