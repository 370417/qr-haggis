import type { Game, GameStage, CardFrontendState } from '../dist/qr_haggis';

import * as React from 'react';
import * as ReactDOM from 'react-dom';


const QR_WIDTH = 296;
const QR_HEIGHT = 296;

import('../dist/qr_haggis').then(module => {

    let game = module.Game.new();

    type AppState = {
        stage: GameStage,
        qrBlob: Blob | null,
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
                qrBlob: null,
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
                            qrBlob: null,
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
                        qrBlob: null,
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
                    qrBlob: blob,
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
            const success = game.from_qr_code(new Uint8Array(imageData));
            if (!success) {
                console.warn("Error reading qr code");
            } else {
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
                        isSelectionEmpty={this.state.selectedCards.size == 0}
                        readQRHandler={this.readQRHandler}
                        buttonHandler={this.buttonHandler}
                        qrBlob={this.state.qrBlob}
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
        qrBlob: Blob | null,
        isSelectionValid: boolean,
        isSelectionEmpty: boolean,
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
            const button = <Button stage={this.props.stage} isSelectionValid={this.props.isSelectionValid} isSelectionEmpty={this.props.isSelectionEmpty} buttonHandler={this.props.buttonHandler} outcome={outcome} />;
            switch (this.props.stage) {
                case module.GameStage.BeforeGame:
                    return <>
                        {button}
                        <QRReader readQRHandler={this.props.readQRHandler} />
                    </>;
                case module.GameStage.Play:
                    return button;
                case module.GameStage.Wait:
                    return <>
                        {this.props.qrBlob !== null ? <QRDisplay qrBlob={this.props.qrBlob} /> : ""}
                        {button}
                        <QRReader readQRHandler={this.props.readQRHandler} />
                    </>;
                case module.GameStage.GameOver:
                    return <>
                        {this.props.qrBlob !== null ? <QRDisplay qrBlob={this.props.qrBlob} /> : ""}
                        {button}
                    </>;
            }
        }
    }

    type QRDisplayProps = {
        qrBlob: Blob,
    };

    class QRDisplay extends React.Component<QRDisplayProps> {
        constructor(props: QRDisplayProps) {
            super(props);

            this.copy = this.copy.bind(this);
        }

        copy() {
            // @ts-ignore
            navigator.clipboard.write([new ClipboardItem({
                [this.props.qrBlob.type]: this.props.qrBlob
            })]);
        }

        render() {
            return <>
                <img id="qr_display" src={URL.createObjectURL(this.props.qrBlob)} />
                <div id="copy_button" onClick={this.copy}><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24"><path fill-rule="evenodd" d="M4.75 3A1.75 1.75 0 003 4.75v9.5c0 .966.784 1.75 1.75 1.75h1.5a.75.75 0 000-1.5h-1.5a.25.25 0 01-.25-.25v-9.5a.25.25 0 01.25-.25h9.5a.25.25 0 01.25.25v1.5a.75.75 0 001.5 0v-1.5A1.75 1.75 0 0014.25 3h-9.5zm5 5A1.75 1.75 0 008 9.75v9.5c0 .966.784 1.75 1.75 1.75h9.5A1.75 1.75 0 0021 19.25v-9.5A1.75 1.75 0 0019.25 8h-9.5zM9.5 9.75a.25.25 0 01.25-.25h9.5a.25.25 0 01.25.25v9.5a.25.25 0 01-.25.25h-9.5a.25.25 0 01-.25-.25v-9.5z"></path></svg></div>
            </>;
        }
    }

    type ButtonProps = {
        stage: GameStage,
        outcome: Outcome,
        isSelectionValid: boolean,
        isSelectionEmpty: boolean,
        buttonHandler: () => void,
    };

    class Button extends React.Component<ButtonProps> {
        render() {
            switch (this.props.stage) {
                case module.GameStage.BeforeGame:
                    return <div id="button" className="enabled" onClick={this.props.buttonHandler}>start</div>;
                case module.GameStage.Play:
                    if (this.props.isSelectionValid) {
                        const text = this.props.isSelectionEmpty ? "pass" : "play";
                        return <div id="button" className="enabled" onClick={this.props.buttonHandler}>{text}</div>;
                    } else {
                        return <div id="button">play</div>;
                    }
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
            this.paste = this.paste.bind(this);
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

        paste() {
            // clipboard-read is correct according to w3c, typescript thinks it's clipboard
            // @ts-ignore
            navigator.permissions.query({ name: "clipboard-read" }).then(result => {
                // If permission to read the clipboard is granted or if the user will
                // be prompted to allow it, we proceed.

                let x: Clipboard;

                if (result.state == "granted" || result.state == "prompt") {
                    // @ts-ignore
                    navigator.clipboard.read().then(items => {
                        console.log(items);
                        items[0].getType("image/png").then((blob: Blob | null) => {
                            if (blob) {
                                console.log(blob);
                                blob?.arrayBuffer().then((arrayBuffer: ArrayBuffer) => {
                                    this.props.readQRHandler(arrayBuffer);
                                });
                            }
                        });
                    });
                }
            });
        }

        render() {
            return <>
                <label id="qr_reader"
                    onDragOver={this.dragOver}
                    onDragEnter={this.dragOver}
                    onDrop={this.drop}>
                    <input
                        type="file"
                        onChange={this.selectFile}
                        style={{ display: "none" }} />
                </label>
                <div id="paste_button" onClick={this.paste}><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24"><path fill-rule="evenodd" d="M5.962 2.513a.75.75 0 01-.475.949l-.816.272a.25.25 0 00-.171.237V21.25c0 .138.112.25.25.25h14.5a.25.25 0 00.25-.25V3.97a.25.25 0 00-.17-.236l-.817-.272a.75.75 0 01.474-1.424l.816.273A1.75 1.75 0 0121 3.97v17.28A1.75 1.75 0 0119.25 23H4.75A1.75 1.75 0 013 21.25V3.97a1.75 1.75 0 011.197-1.66l.816-.272a.75.75 0 01.949.475z"></path><path fill-rule="evenodd" d="M7 1.75C7 .784 7.784 0 8.75 0h6.5C16.216 0 17 .784 17 1.75v1.5A1.75 1.75 0 0115.25 5h-6.5A1.75 1.75 0 017 3.25v-1.5zm1.75-.25a.25.25 0 00-.25.25v1.5c0 .138.112.25.25.25h6.5a.25.25 0 00.25-.25v-1.5a.25.25 0 00-.25-.25h-6.5z"></path></svg></div>
            </>;
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

