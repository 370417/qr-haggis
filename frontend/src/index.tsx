import type { Game, GameStage, CardFrontendState } from '../dist/qr_haggis';

import * as React from 'react';
import * as ReactDOM from 'react-dom';
import Dropzone from 'react-dropzone';


const QR_WIDTH = 296;
const QR_HEIGHT = 296;

import('../dist/qr_haggis').then(module => {

    const game = module.Game.new();

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
            const player = game.am_player_1() ? "player1" : "player2";
            return (
                <div id="app" className={`${player} stage${this.state.stage}`}>
                    <CardGrid
                        stage={this.state.stage}
                        selectedCards={this.state.selectedCards}
                        cardHandler={this.cardHandler}
                        game={game} />
                    <Scores
                        myScore={scores[0]}
                        opponentScore={scores[1]} />
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
            switch (this.props.stage) {
                case module.GameStage.BeforeGame:
                    return <>
                        <Button stage={this.props.stage} isSelectionValid={this.props.isSelectionValid} buttonHandler={this.props.buttonHandler} myScore={this.props.myScore} opponentScore={this.props.opponentScore} />
                        <QRReader2 readQRHandler={this.props.readQRHandler} />
                    </>;
                case module.GameStage.Play:
                    return <Button stage={this.props.stage} isSelectionValid={this.props.isSelectionValid} buttonHandler={this.props.buttonHandler} myScore={this.props.myScore} opponentScore={this.props.opponentScore} />
                case module.GameStage.Wait:
                    return <>
                        {this.props.qrObjectUrl !== undefined ? <QRDisplay qrObjectUrl={this.props.qrObjectUrl} /> : ""}
                        <Button stage={this.props.stage} isSelectionValid={this.props.isSelectionValid} buttonHandler={this.props.buttonHandler} myScore={this.props.myScore} opponentScore={this.props.opponentScore} />
                        <QRReader2 readQRHandler={this.props.readQRHandler} />
                    </>;
                case module.GameStage.GameOver:
                    return <>
                        {this.props.qrObjectUrl !== undefined ? <QRDisplay qrObjectUrl={this.props.qrObjectUrl} /> : ""}
                        <Button stage={this.props.stage} isSelectionValid={this.props.isSelectionValid} buttonHandler={this.props.buttonHandler} myScore={this.props.myScore} opponentScore={this.props.opponentScore} />
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
        myScore: number,
        opponentScore: number,
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

    type QRReaderProps = {
        readQRHandler: (imageData: ArrayBuffer) => void,
    };

    class QRReader extends React.Component<QRReaderProps> {
        constructor(props: QRReaderProps) {
            super(props);
            this.onDrop = this.onDrop.bind(this);
        }

        onDrop(acceptedFiles: File[]) {
            console.log("In onDrop");

            const file = acceptedFiles[0];
            const reader = new FileReader();

            const readQRHandler = this.props.readQRHandler;
            reader.onload = () => {
                readQRHandler(reader.result as ArrayBuffer);
            };
            reader.readAsArrayBuffer(file);
        }

        render() {
            return <Dropzone onDrop={this.onDrop}>
                {({ getRootProps, getInputProps }) => (
                    <div id="qr_reader2" {...getRootProps()}>
                        <input {...getInputProps()} />
                        <p>Drag 'n' drop some files here, or click to select files</p>
                    </div>
                )}
            </Dropzone>;
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

    class QRReader2 extends React.Component<QRReaderProps> {
        dragOver(event: React.DragEvent) {
            if (event.dataTransfer.types.includes("text/uri-list")) {
                event.dataTransfer.dropEffect = "copy";
                event.preventDefault();
            }
        }

        drop(event: React.DragEvent) {
            event.preventDefault();

            if (event.dataTransfer.types.includes("text/uri-list")) {
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
            }
        }

        render() {
            return <div id="qr_reader"
                onDragOver={this.dragOver}
                onDragEnter={this.dragOver}
                onDrop={this.drop.bind(this)}></div>;
        }
    }

    ReactDOM.render(<App />, document.body);
});

