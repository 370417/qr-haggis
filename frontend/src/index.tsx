import type { Game, GameStage, CardFrontendState } from '../dist/qr_haggis';

import * as React from 'react';
import * as ReactDOM from 'react-dom';

const QR_WIDTH = 296;
const QR_HEIGHT = 296;

import('../dist/qr_haggis').then(module => {

    type AppState = {
        stage: GameStage,
        myScore: number,
        opponentScore: number,
        qrPixels: Uint8Array,
        game: Game,
        canPlay: boolean,
        selectedCards: Set<number>,
    };

    class App extends React.Component<{}, AppState> {
        constructor() {
            super({});
            this.state = {
                stage: module.GameStage.BeforeGame,
                myScore: 0,
                opponentScore: 0,
                qrPixels: Uint8Array.from([]),
                game: module.Game.new(),
                canPlay: false,
                selectedCards: new Set(),
            };
            this.buttonHandler = this.buttonHandler.bind(this);
            this.cardHandler = this.cardHandler.bind(this);
        }

        buttonHandler() {
            switch (this.state.stage) {
                case module.GameStage.BeforeGame:
                    this.setState({
                        stage: module.GameStage.Play,
                    });
                    break;
                case module.GameStage.Play:
                    if (this.state.canPlay) {
                        this.state.game.play_cards(Uint32Array.from(this.state.selectedCards));
                        let scores = this.state.game.calculate_score();
                        let stage = this.state.game.game_stage();
                        let qrPixels = this.state.game.to_qr_code(QR_WIDTH, QR_HEIGHT);
                        this.setState({
                            stage,
                            myScore: scores[0],
                            opponentScore: scores[1],
                            canPlay: false,
                            selectedCards: new Set(),
                            qrPixels,
                        });
                    } else {
                        alert("You did not select a valid card combination.");
                    }
                    break;
            }
        }

        cardHandler(cardId: number) {
            if (this.state.selectedCards.has(cardId)) {
                this.state.selectedCards.delete(cardId);
            } else {
                this.state.selectedCards.add(cardId);
            }
            let canPlay = this.state.game.can_play_cards(Uint32Array.from(this.state.selectedCards));
            this.setState({
                selectedCards: new Set(this.state.selectedCards),
                canPlay,
            });
        }

        render() {
            return (
                <div id="app">
                    <CardGrid stage={this.state.stage} selectedCards={this.state.selectedCards} cardHandler={this.cardHandler} game={this.state.game} />
                    <Sidebar stage={this.state.stage} canPlay={this.state.canPlay} buttonHandler={this.buttonHandler} qrPixels={this.state.qrPixels} myScore={this.state.myScore} opponentScore={this.state.opponentScore} />
                    <Scores myScore={this.state.myScore} opponentScore={this.state.opponentScore} />
                </div>
            );
        }
    }

    type CardGridProps = {
        stage: GameStage,
        game: Game,
        selectedCards: Set<number>,
        cardHandler: (cardId: number) => void,
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
                normalCards.push(<Card
                    key={key}
                    cardHandler={this.props.cardHandler}
                    frontendState={this.props.game.card_frontend_state(key)}
                    cardId={key}
                    selected={this.props.selectedCards.has(key)}
                    stage={this.props.stage} />);
            }
            for (let key = 36; key < 39; key++) {
                myWilcards.push(<Card key={key}
                    cardHandler={this.props.cardHandler}
                    frontendState={this.props.game.card_frontend_state(key)}
                    cardId={key}
                    selected={this.props.selectedCards.has(key)}
                    stage={this.props.stage} />);
            }
            for (let key = 39; key < 42; key++) {
                opponentWildcards.push(<Card key={key}
                    cardHandler={this.props.cardHandler}
                    frontendState={this.props.game.card_frontend_state(key)}
                    cardId={key}
                    selected={this.props.selectedCards.has(key)}
                    stage={this.props.stage} />);
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

    type CardProps = {
        cardId: number,
        frontendState: CardFrontendState,
        selected: boolean,
        stage: GameStage,
        cardHandler: (cardId: number) => void,
    };

    class Card extends React.Component<CardProps> {
        render() {
            if (this.props.stage == module.GameStage.BeforeGame) {
                return <div className="card"></div>;
            }
            let className = "card";
            if (this.props.selected) {
                className += " selected";
            }
            className += ` ${this.props.frontendState}`;

            if (this.props.stage == module.GameStage.Play && this.props.frontendState == module.CardFrontendState.InMyHand) {
                return <div className={className} onClick={() => this.props.cardHandler(this.props.cardId)}>{this.props.frontendState}</div>;
            }

            return <div className={className}>{this.props.frontendState}</div>;
        }
    }

    type SidebarProps = {
        stage: GameStage,
        myScore: number,
        opponentScore: number,
        qrPixels: Uint8Array,
        canPlay: boolean,
        buttonHandler: () => void,
    };

    class Sidebar extends React.Component<SidebarProps> {
        render() {
            switch (this.props.stage) {
                case module.GameStage.BeforeGame:
                    return <>
                        <Button stage={this.props.stage} canPlay={this.props.canPlay} buttonHandler={this.props.buttonHandler} myScore={this.props.myScore} opponentScore={this.props.opponentScore} />
                        <QRReader />
                    </>;
                case module.GameStage.Play:
                    return <Button stage={this.props.stage} canPlay={this.props.canPlay} buttonHandler={this.props.buttonHandler} myScore={this.props.myScore} opponentScore={this.props.opponentScore} />
                case module.GameStage.Wait:
                    return <>
                        <QRDisplay qrPixels={this.props.qrPixels} />
                        <Button stage={this.props.stage} canPlay={this.props.canPlay} buttonHandler={this.props.buttonHandler} myScore={this.props.myScore} opponentScore={this.props.opponentScore} />
                        <QRReader />
                    </>;
                case module.GameStage.GameOver:
                    return <>
                        <QRDisplay qrPixels={this.props.qrPixels} />
                        <Button stage={this.props.stage} canPlay={this.props.canPlay} buttonHandler={this.props.buttonHandler} myScore={this.props.myScore} opponentScore={this.props.opponentScore} />
                    </>;
            }
        }
    }

    type QRDisplayProps = {
        qrPixels: Uint8Array,
    };

    class QRDisplay extends React.Component<QRDisplayProps> {
        render() {
            let canvas = document.createElement('canvas');
            let clampedArray = Uint8ClampedArray.from(this.props.qrPixels);
            let size = Math.sqrt(clampedArray.length / 4);
            canvas.width = size;
            canvas.height = size;
            let ctx = canvas.getContext('2d');
            if (ctx) {
                console.log(clampedArray, canvas);
                let imageData = new ImageData(clampedArray, canvas.width, canvas.height);
                ctx.putImageData(imageData, 0, 0);
            }
            return <img id="qr_display" src={canvas.toDataURL()} />;
        }
    }

    type ButtonProps = {
        stage: GameStage,
        myScore: number,
        opponentScore: number,
        canPlay: boolean,
        buttonHandler: () => void,
    };

    class Button extends React.Component<ButtonProps> {
        render() {
            switch (this.props.stage) {
                case module.GameStage.BeforeGame:
                    return <div id="button" className="enabled" onClick={this.props.buttonHandler}>start</div>;
                case module.GameStage.Play:
                    if (this.props.canPlay) {
                        return <div id="button" className="enabled" onClick={this.props.buttonHandler}>play</div>;
                    }
                    return <div id="button">play</div>;
                case module.GameStage.Wait:
                    return <div id="button">wait</div>;
                case module.GameStage.GameOver:
                    if (this.props.myScore > this.props.opponentScore) {
                        return <div id="button">you won</div>;
                    } else if (this.props.myScore < this.props.opponentScore) {
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
        myScore: number,
        opponentScore: number,
    };

    class Scores extends React.Component<ScoresProps> {
        render() {
            return <div id="scores">{this.props.myScore}pts / {this.props.opponentScore}pts</div>;
        }
    }

    ReactDOM.render(<App />, document.body);
});

