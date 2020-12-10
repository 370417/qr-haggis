import type { Game, GameStage, CardFrontendState } from '../dist/qr_haggis';

import * as React from 'react';
import * as ReactDOM from 'react-dom';


const QR_WIDTH = 296;
const QR_HEIGHT = 296;

import('../dist/qr_haggis').then(module => {

    let game = module.Game.new();

    type AppState = {
        stage: GameStage,
        qrObjectUrl: string | undefined,
        myScore: number,
        opponentScore: number,

        isSelectionValid: boolean,
        selectedCards: Set<number>,
    };

    class App extends React.Component<{}, AppState> {
        constructor() {
            super({});
            this.state = {
                stage: module.GameStage.BeforeGame,
                qrObjectUrl: undefined,
                myScore: 0,
                opponentScore: 0,
                isSelectionValid: false,
                selectedCards: new Set(),
            };
            this.buttonHandler = this.buttonHandler.bind(this);
            this.cardHandler = this.cardHandler.bind(this);
            this.readQRHandler = this.readQRHandler.bind(this);
        }

        buttonHandler() {
            switch (this.state.stage) {
                case module.GameStage.BeforeGame:
                    this.setState({
                        stage: module.GameStage.Play,
                    });
                    break;
                case module.GameStage.Play:
                    if (this.state.isSelectionValid) {
                        game.play_cards(Uint32Array.from(this.state.selectedCards));
                        let scores = game.calculate_score();

                        this.setState({
                            stage: game.game_stage(),
                            selectedCards: new Set(),
                            myScore: scores[0],
                            opponentScore: scores[1],
                            qrObjectUrl: undefined,
                        });

                        this.renderQRCode();
                    } else {
                        alert("You did not select a valid card combination.");
                    }
                    break;
                case module.GameStage.GameOver:
                    game = module.Game.new();
                    this.setState({
                        stage: module.GameStage.BeforeGame,
                        qrObjectUrl: undefined,
                        myScore: 0,
                        opponentScore: 0,
                        isSelectionValid: false,
                        selectedCards: new Set(),
                    });
                    break;
            }
        }

        renderQRCode() {
            let qrPixels = game.to_qr_code(QR_WIDTH, QR_HEIGHT);

            let canvas = document.createElement('canvas');
            let size = Math.sqrt(qrPixels.length / 4);
            canvas.width = size;
            canvas.height = size;

            let ctx = canvas.getContext('2d');
            let imageData = new ImageData(qrPixels, canvas.width, canvas.height);
            ctx?.putImageData(imageData, 0, 0);

            canvas.toBlob(blob => {
                this.setState({
                    qrObjectUrl: URL.createObjectURL(blob),
                });
            });
        }

        cardHandler(cardId: number) {
            if (this.state.selectedCards.has(cardId)) {
                this.state.selectedCards.delete(cardId);
            } else {
                this.state.selectedCards.add(cardId);
            }
            let isSelectionValid = game.can_play_cards(Uint32Array.from(this.state.selectedCards));
            this.setState({
                selectedCards: new Set(this.state.selectedCards),
                isSelectionValid,
            });
        }

        readQRHandler(imageData: ArrayBuffer) {
            game.from_qr_code(new Uint8Array(imageData));
            let scores = game.calculate_score();
            console.log(game.game_stage());
            this.setState({
                stage: game.game_stage(),
                selectedCards: new Set(),
                myScore: scores[0],
                opponentScore: scores[1],
                isSelectionValid: game.can_play_cards(Uint32Array.from([])),
            });
        }

        render() {
            const scores = game.calculate_score();
            const handSizes = game.hand_sizes();
            let player: string, player1score: number, player2score: number, player1hand, player2hand;
            if (game.am_player_1()) {
                player = "player1";
                player1score = scores[0];
                player2score = scores[1];
                player1hand = handSizes[0];
                player2hand = handSizes[1];
            } else {
                player = "player2";
                player1score = scores[1];
                player2score = scores[0];
                player1hand = handSizes[1];
                player2hand = handSizes[0];
            }
            return (
                <div id="app" className={`${player} stage${this.state.stage}`}>
                    <HandSizes
                        player1hand={player1hand}
                        player2hand={player2hand} />
                    <CardGrid
                        stage={this.state.stage}
                        selectedCards={this.state.selectedCards}
                        cardHandler={this.cardHandler}
                        game={game} />
                    <Scores
                        player1score={player1score}
                        player2score={player2score} />
                    <Sidebar stage={this.state.stage}
                        isSelectionValid={this.state.isSelectionValid}
                        readQRHandler={this.readQRHandler}
                        buttonHandler={this.buttonHandler}
                        qrObjectUrl={this.state.qrObjectUrl}
                        myScore={this.state.myScore}
                        opponentScore={this.state.opponentScore} />
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
            let wildcardLabels = ['J', 'Q', 'K'];
            for (let rank = 2; rank <= 10; rank++) {
                rankLabels.push(<span>{rank}</span>);
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
                <div id="suits"><span>♠</span><span>♥</span><span>♦</span><span>♣</span></div>
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
                return <div className="card state0"></div>;
            }

            let className = "card";
            if (this.props.selected) {
                className += " selected";
            }
            className += ` state${this.props.frontendState}`;

            if (this.props.stage == module.GameStage.Play && this.props.frontendState == module.CardFrontendState.InMyHand) {
                return <div className={className} onClick={() => this.props.cardHandler(this.props.cardId)}></div>;
            }

            return <div className={className}></div>;
        }
    }

    enum Outcome {
        Won,
        Lost,
        Tied,
    }

    type SidebarProps = {
        stage: GameStage,
        myScore: number,
        opponentScore: number,
        qrObjectUrl: string | undefined,
        isSelectionValid: boolean,
        buttonHandler: () => void,
        readQRHandler: (imageData: ArrayBuffer) => void,
    };

    class Sidebar extends React.Component<SidebarProps> {
        render() {
            let outcome = Outcome.Tied;
            if (this.props.myScore > this.props.opponentScore) {
                outcome = Outcome.Won;
            } else if (this.props.myScore < this.props.opponentScore) {
                outcome = Outcome.Lost;
            }
            switch (this.props.stage) {
                case module.GameStage.BeforeGame:
                    return <>
                        <Button stage={this.props.stage} isSelectionValid={this.props.isSelectionValid} buttonHandler={this.props.buttonHandler} outcome={outcome} />
                        <QRReader readQRHandler={this.props.readQRHandler} />
                    </>;
                case module.GameStage.Play:
                    return <Button stage={this.props.stage} isSelectionValid={this.props.isSelectionValid} buttonHandler={this.props.buttonHandler} outcome={outcome} />
                case module.GameStage.Wait:
                    return <>
                        {this.props.qrObjectUrl !== undefined ? <QRDisplay qrObjectUrl={this.props.qrObjectUrl} /> : ""}
                        <Button stage={this.props.stage} isSelectionValid={this.props.isSelectionValid} buttonHandler={this.props.buttonHandler} outcome={outcome} />
                        <QRReader readQRHandler={this.props.readQRHandler} />
                    </>;
                case module.GameStage.GameOver:
                    return <>
                        {this.props.qrObjectUrl !== undefined ? <QRDisplay qrObjectUrl={this.props.qrObjectUrl} /> : ""}
                        <Button stage={this.props.stage} isSelectionValid={this.props.isSelectionValid} buttonHandler={this.props.buttonHandler} outcome={outcome} />
                    </>;
            }
        }
    }

    type QRDisplayProps = {
        qrObjectUrl: string,
    };

    class QRDisplay extends React.Component<QRDisplayProps> {
        render() {
            return <img id="qr_display" src={this.props.qrObjectUrl} />;
        }
    }

    type ButtonProps = {
        stage: GameStage,
        outcome: Outcome,
        isSelectionValid: boolean,
        buttonHandler: () => void,
    };

    class Button extends React.Component<ButtonProps> {
        render() {
            switch (this.props.stage) {
                case module.GameStage.BeforeGame:
                    return <div id="button" className="enabled" onClick={this.props.buttonHandler}>start</div>;
                case module.GameStage.Play:
                    if (this.props.isSelectionValid) {
                        return <div id="button" className="enabled" onClick={this.props.buttonHandler}>play</div>;
                    }
                    return <div id="button">play</div>;
                case module.GameStage.Wait:
                    return <div id="button">wait</div>;
                case module.GameStage.GameOver:
                    switch (this.props.outcome) {
                        case Outcome.Won:
                            return <div id="button" className="won" onClick={this.props.buttonHandler}>you won!</div>;
                        case Outcome.Lost:
                            return <div id="button" className="lost" onClick={this.props.buttonHandler}>you lost.</div>;
                        case Outcome.Tied:
                            return <div id="button" className="tied" onClick={this.props.buttonHandler}>you tied.</div>;
                    }
            }
        }
    }

    type QRReaderProps = {
        readQRHandler: (imageData: ArrayBuffer) => void,
    };

    class QRReader extends React.Component<QRReaderProps> {
        constructor(props: QRReaderProps) {
            super(props);

            this.dragOver = this.dragOver.bind(this);
            this.drop = this.drop.bind(this);
            this.selectFile = this.selectFile.bind(this);
        }

        dragOver(event: React.DragEvent) {
            if (event.dataTransfer.types.includes("text/uri-list")
                || event.dataTransfer.items[0].kind == "file"
            ) {
                event.dataTransfer.dropEffect = "copy";
                event.preventDefault();
            }
        }

        drop(event: React.DragEvent) {
            event.preventDefault();

            if (event.dataTransfer.types.includes("text/uri-list")) {
                // Dragging from browser windows
                const uri = event.dataTransfer.getData("text/uri-list");

                const img = new Image();
                img.crossOrigin = "Anonymous";
                img.onload = () => {
                    const canvas = document.createElement("canvas");
                    canvas.width = img.naturalWidth;
                    canvas.height = img.naturalHeight;
                    canvas.getContext("2d")?.drawImage(img, 0, 0);

                    canvas.toBlob(blob => {
                        blob?.arrayBuffer().then(arrayBuffer => {
                            this.props.readQRHandler(arrayBuffer);
                        });
                    });
                };
                img.src = uri;
            } else if (event.dataTransfer.items[0]) {
                // Dragging from the filesystem
                const file = event.dataTransfer.items[0].getAsFile();
                if (!file) {
                    return;
                }
                createImageBitmap(file).then(bitmap => {
                    const canvas = document.createElement("canvas");
                    canvas.width = bitmap.width;
                    canvas.height = bitmap.height;
                    canvas.getContext("2d")?.drawImage(bitmap, 0, 0);

                    canvas.toBlob(blob => {
                        blob?.arrayBuffer().then(arrayBuffer => {
                            this.props.readQRHandler(arrayBuffer);
                        });
                    });
                });
            }
        }

        selectFile(event: React.ChangeEvent) {
            const element = event.target as HTMLInputElement;

            const file = element.files && element.files[0];
            if (file) {
                createImageBitmap(file).then(bitmap => {
                    const canvas = document.createElement("canvas");
                    canvas.width = bitmap.width;
                    canvas.height = bitmap.height;
                    canvas.getContext("2d")?.drawImage(bitmap, 0, 0);

                    canvas.toBlob(blob => {
                        blob?.arrayBuffer().then(arrayBuffer => {
                            this.props.readQRHandler(arrayBuffer);
                        });
                    });
                });
            }

            // clear the file from the input
            element.value = "";
        }

        render() {
            return <label id="qr_reader"
                onDragOver={this.dragOver}
                onDragEnter={this.dragOver}
                onDrop={this.drop}>
                <input
                    type="file"
                    onChange={this.selectFile}
                    style={{ display: "none" }} />
            </label>;
        }
    }

    type ScoresProps = {
        player1score: number,
        player2score: number,
    };

    class Scores extends React.Component<ScoresProps> {
        render() {
            return <>
                <span id="player1score">{this.props.player1score}{this.props.player1score == 1 ? "pt" : "pts"}</span>
                <span id="player2score">{this.props.player2score}{this.props.player2score == 1 ? "pt" : "pts"}</span>
            </>;
        }
    }

    type HandSizesProps = {
        player1hand: number,
        player2hand: number,
    };

    class HandSizes extends React.Component<HandSizesProps> {
        render() {
            return <div id="hand_sizes">
                <span id="player1hand">{this.props.player1hand}</span>
                <span id="hand_separator">–</span>
                <span id="player2hand">{this.props.player2hand}</span>
            </div>;
        }
    }

    ReactDOM.render(<App />, document.body);
});

