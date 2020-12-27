import type { Game, GameStage, CardFrontendState } from "../dist/qr_haggis";

import * as React from "react";
import * as ReactDOM from "react-dom";

const QR_WIDTH = 296;
const QR_HEIGHT = 296;

import("../dist/qr_haggis").then((module) => {
  let game = module.Game.new();

  type AppState = {
    stage: GameStage;
    // Store qr code for copying
    outputQrBlob: Blob | null;
    // Store url to qr code to display in img
    outputQrObjectUrl: string | null;
    myScore: number;
    opponentScore: number;

    isSelectionValid: boolean;
    // Cardids of the selected cards
    selectedCards: Set<number>;

    websocket: WebSocket | null;
  };

  class App extends React.Component<{}, AppState> {
    constructor() {
      super({});
      this.state = {
        stage: module.GameStage.BeforeGame,
        outputQrBlob: null,
        outputQrObjectUrl: null,
        myScore: 0,
        opponentScore: 0,
        isSelectionValid: false,
        selectedCards: new Set(),
        websocket: null,
      };
      this.buttonHandler = this.buttonHandler.bind(this);
      this.cardClickHandler = this.cardClickHandler.bind(this);
      this.qrReadHandler = this.qrReadHandler.bind(this);
    }

    // Update the game after a move has been made and close the websocket
    // if the game is over.
    // To set a new websocket, include a newWebsocket argument.
    updateGame(newWebsocket?: WebSocket) {
      const scores = game.calculate_score();
      const stage = game.game_stage();

      let websocket = newWebsocket || this.state.websocket;
      if (stage == module.GameStage.GameOver) {
        this.state.websocket?.close();
        websocket = null;
      }

      this.setState({
        stage,
        selectedCards: new Set(),
        myScore: scores[0],
        opponentScore: scores[1],
        outputQrBlob: null,
        outputQrObjectUrl: null,
        isSelectionValid: game.can_play_cards(Uint32Array.from([])),
        websocket,
      });
    }

    // Create a websocket connection to the sever for the duration of this game
    createWebsocket(): WebSocket {
      const websocket = new WebSocket("wss://qr-haggis.herokuapp.com/v1");
      websocket.binaryType = "arraybuffer";

      // Update the game when receiving messages
      websocket.addEventListener("message", (event) => {
        const array = new Uint8Array(event.data);
        if (array) {
          const success = game.from_compressed(array);
          if (!success) {
            console.warn("Error reading data from server");
          } else {
            this.updateGame();
          }
        }
      });

      // The server expects the first message to be the client id
      websocket.addEventListener("open", () => {
        websocket.send(game.get_client_id());
      });

      return websocket;
    }

    // Handle the user clicking on the sidebar button
    buttonHandler() {
      switch (this.state.stage) {
        // Start the game
        case module.GameStage.BeforeGame:
          this.setState({
            stage: module.GameStage.Play,
            websocket: this.createWebsocket(),
          });
          break;
        // Play the selected cards
        case module.GameStage.Play:
          if (this.state.isSelectionValid) {
            game.play_cards(Uint32Array.from(this.state.selectedCards));

            this.state.websocket?.send(game.to_compressed());
            this.updateGame();
            this.renderOutputQRCode();
          } else {
            alert("You did not select a valid card combination.");
          }
          break;
        // Reset App to BeforeGame
        case module.GameStage.GameOver:
          game = module.Game.new();
          this.setState({
            stage: module.GameStage.BeforeGame,
            outputQrBlob: null,
            myScore: 0,
            opponentScore: 0,
            isSelectionValid: false,
            selectedCards: new Set(),
          });
          break;
      }
    }

    // Asynchronously generate a qr code representing the current game state
    // and update this.state.outputQrBlob/this.state.outputQrObjectUrl
    renderOutputQRCode() {
      const qrPixels = game.to_qr_code(QR_WIDTH, QR_HEIGHT);

      const canvas = document.createElement("canvas");
      // Each pixel gets four bytes (rgba)
      const size = Math.sqrt(qrPixels.length / 4);
      canvas.width = size;
      canvas.height = size;

      const ctx = canvas.getContext("2d");
      const imageData = new ImageData(qrPixels, canvas.width, canvas.height);
      ctx?.putImageData(imageData, 0, 0);

      if (this.state.outputQrObjectUrl) {
        // Allow old outputQrBlob to be freed
        URL.revokeObjectURL(this.state.outputQrObjectUrl);
      }

      canvas.toBlob((blob) => {
        this.setState({
          outputQrBlob: blob,
          outputQrObjectUrl: URL.createObjectURL(blob),
        });
      });
    }

    // Handle the user clicking on a card when game stage is Play
    cardClickHandler(cardId: number) {
      if (this.state.selectedCards.has(cardId)) {
        this.state.selectedCards.delete(cardId);
      } else {
        this.state.selectedCards.add(cardId);
      }
      const isSelectionValid = game.can_play_cards(
        Uint32Array.from(this.state.selectedCards)
      );
      this.setState({
        selectedCards: new Set(this.state.selectedCards),
        isSelectionValid,
      });
    }

    // Update the game based on an input qr image and create a new websocket
    // connection if none exists
    qrReadHandler(imageData: ArrayBuffer) {
      const success = game.from_qr_code(new Uint8Array(imageData));
      if (!success) {
        console.warn("Error reading qr code");
      } else if (this.state.websocket === null) {
        this.updateGame(this.createWebsocket());
      } else {
        this.updateGame();
      }
    }

    render() {
      const scores = game.calculate_score();
      const handSizes = game.hand_sizes();
      // Organize player data by first/second player instead of by me/opponent
      // so that the two players are assigned different colors and sides
      // of the board for wildcards
      let playerClassName: string,
        firstPlayerScore: number,
        secondPlayerScore: number,
        firstPlayerHandSize,
        secondPlayerHandSize;
      if (game.me_went_first()) {
        playerClassName = "firstPlayer";
        firstPlayerScore = scores[0];
        secondPlayerScore = scores[1];
        firstPlayerHandSize = handSizes[0];
        secondPlayerHandSize = handSizes[1];
      } else {
        playerClassName = "secondPlayer";
        firstPlayerScore = scores[1];
        secondPlayerScore = scores[0];
        firstPlayerHandSize = handSizes[1];
        secondPlayerHandSize = handSizes[0];
      }
      return (
        <div id="app" className={`${playerClassName} stage${this.state.stage}`}>
          <HandSizes
            firstPlayerHandSize={firstPlayerHandSize}
            secondPlayerHandSize={secondPlayerHandSize}
          />
          <CardGrid
            stage={this.state.stage}
            selectedCards={this.state.selectedCards}
            cardClickHandler={this.cardClickHandler}
            game={game}
          />
          <Scores
            firstPlayerScore={firstPlayerScore}
            secondPlayerScore={secondPlayerScore}
          />
          <Sidebar
            stage={this.state.stage}
            isSelectionValid={this.state.isSelectionValid}
            isSelectionEmpty={this.state.selectedCards.size == 0}
            qrReadHandler={this.qrReadHandler}
            buttonHandler={this.buttonHandler}
            outputQrBlob={this.state.outputQrBlob}
            outputQrObjectUrl={this.state.outputQrObjectUrl}
            myScore={this.state.myScore}
            opponentScore={this.state.opponentScore}
          />
        </div>
      );
    }
  }

  type CardGridProps = {
    stage: GameStage;
    game: Game;
    selectedCards: Set<number>;
    cardClickHandler: (cardId: number) => void;
  };

  // Contains all the cards and labels for suits and ranks
  class CardGrid extends React.Component<CardGridProps> {
    render() {
      const rankLabels = [];
      const wildcardLabels = ["J", "Q", "K"];
      for (let rank = 2; rank <= 10; rank++) {
        rankLabels.push(<span>{rank}</span>);
      }
      const normalCards = [];
      const myWilcards = [];
      const opponentWildcards = [];
      for (let key = 0; key < 36; key++) {
        normalCards.push(
          <Card
            key={key}
            cardClickHandler={this.props.cardClickHandler}
            frontendState={this.props.game.card_frontend_state(key)}
            cardId={key}
            selected={this.props.selectedCards.has(key)}
            stage={this.props.stage}
          />
        );
      }
      for (let key = 36; key < 39; key++) {
        myWilcards.push(
          <Card
            key={key}
            cardClickHandler={this.props.cardClickHandler}
            frontendState={this.props.game.card_frontend_state(key)}
            cardId={key}
            selected={this.props.selectedCards.has(key)}
            stage={this.props.stage}
          />
        );
      }
      for (let key = 39; key < 42; key++) {
        opponentWildcards.push(
          <Card
            key={key}
            cardClickHandler={this.props.cardClickHandler}
            frontendState={this.props.game.card_frontend_state(key)}
            cardId={key}
            selected={this.props.selectedCards.has(key)}
            stage={this.props.stage}
          />
        );
      }
      return (
        <div id="card_grid">
          <div id="ranks">
            <span></span>
            {rankLabels}
          </div>
          <div id="suits">
            <span>♠</span>
            <span>♥</span>
            <span>♦</span>
            <span>♣</span>
          </div>
          <div id="normal_cards">{normalCards}</div>
          <div id="my_wildcards">
            {myWilcards}
            {wildcardLabels.map((label) => (
              <span>{label}</span>
            ))}
          </div>
          <div id="opponent_wildcards">
            {opponentWildcards}
            {wildcardLabels.map((label) => (
              <span>{label}</span>
            ))}
          </div>
        </div>
      );
    }
  }

  type CardProps = {
    cardId: number;
    frontendState: CardFrontendState;
    selected: boolean;
    stage: GameStage;
    cardClickHandler: (cardId: number) => void;
  };

  // Represents one card; shows whether is selected and shows frontEndState
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

      if (
        this.props.stage == module.GameStage.Play &&
        this.props.frontendState == module.CardFrontendState.InMyHand
      ) {
        return (
          <div
            className={className}
            onClick={() => this.props.cardClickHandler(this.props.cardId)}
          ></div>
        );
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
    stage: GameStage;
    myScore: number;
    opponentScore: number;
    outputQrBlob: Blob | null;
    outputQrObjectUrl: string | null;
    isSelectionValid: boolean;
    isSelectionEmpty: boolean;
    buttonHandler: () => void;
    qrReadHandler: (imageData: ArrayBuffer) => void;
  };

  // Holds QRDisplay, the sidebar button, and QRReader and shows them only when relevant
  class Sidebar extends React.Component<SidebarProps> {
    render() {
      // Don't show QR display while qr blob is still being created asynchronously
      const qrDisplay =
        this.props.outputQrBlob !== null &&
        this.props.outputQrObjectUrl !== null ? (
          <QRDisplay
            outputQrBlob={this.props.outputQrBlob}
            outputQrObjectUrl={this.props.outputQrObjectUrl}
          />
        ) : (
          <></>
        );

      let outcome = Outcome.Tied;
      if (this.props.myScore > this.props.opponentScore) {
        outcome = Outcome.Won;
      } else if (this.props.myScore < this.props.opponentScore) {
        outcome = Outcome.Lost;
      }
      const button = (
        <Button
          stage={this.props.stage}
          isSelectionValid={this.props.isSelectionValid}
          isSelectionEmpty={this.props.isSelectionEmpty}
          buttonHandler={this.props.buttonHandler}
          outcome={outcome}
        />
      );
      const qrReader = (
        <QRReader
          outputQrObjectUrl={this.props.outputQrObjectUrl}
          qrReadHandler={this.props.qrReadHandler}
        />
      );

      switch (this.props.stage) {
        case module.GameStage.BeforeGame:
          return (
            <>
              {button}
              {qrReader}
            </>
          );
        case module.GameStage.Play:
          return button;
        case module.GameStage.Wait:
          return (
            <>
              {qrDisplay}
              {button}
              {qrReader}
            </>
          );
        case module.GameStage.GameOver:
          return (
            <>
              {qrDisplay}
              {button}
            </>
          );
      }
    }
  }

  type QRDisplayProps = {
    outputQrBlob: Blob;
    outputQrObjectUrl: string;
  };

  // Shows the output QR code and allows user to drag or copy it
  class QRDisplay extends React.Component<QRDisplayProps> {
    constructor(props: QRDisplayProps) {
      super(props);

      this.copy = this.copy.bind(this);
    }

    // Copy the qr code as a png blob to the clipboard (supported only in Chrome)
    copy() {
      try {
        // @ts-ignore
        navigator.clipboard.write([
          // @ts-ignore
          new ClipboardItem({
            [this.props.outputQrBlob.type]: this.props.outputQrBlob,
          }),
        ]);
      } catch (e) {
        console.error(e);
      }
    }

    render() {
      return (
        <>
          <img id="qr_display" src={this.props.outputQrObjectUrl} />
          <div id="copy_button" onClick={this.copy}>
            {/* Copy svg icon */}
            <svg
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 24 24"
              width="24"
              height="24"
            >
              <path
                fillRule="evenodd"
                d="M4.75 3A1.75 1.75 0 003 4.75v9.5c0 .966.784 1.75 1.75 1.75h1.5a.75.75 0 000-1.5h-1.5a.25.25 0 01-.25-.25v-9.5a.25.25 0 01.25-.25h9.5a.25.25 0 01.25.25v1.5a.75.75 0 001.5 0v-1.5A1.75 1.75 0 0014.25 3h-9.5zm5 5A1.75 1.75 0 008 9.75v9.5c0 .966.784 1.75 1.75 1.75h9.5A1.75 1.75 0 0021 19.25v-9.5A1.75 1.75 0 0019.25 8h-9.5zM9.5 9.75a.25.25 0 01.25-.25h9.5a.25.25 0 01.25.25v9.5a.25.25 0 01-.25.25h-9.5a.25.25 0 01-.25-.25v-9.5z"
              ></path>
            </svg>
          </div>
        </>
      );
    }
  }

  type ButtonProps = {
    stage: GameStage;
    outcome: Outcome;
    isSelectionValid: boolean;
    isSelectionEmpty: boolean;
    buttonHandler: () => void;
  };

  // Button to start the game or play cards
  class Button extends React.Component<ButtonProps> {
    render() {
      switch (this.props.stage) {
        case module.GameStage.BeforeGame:
          return (
            <div
              id="button"
              className="enabled"
              onClick={this.props.buttonHandler}
            >
              start
            </div>
          );
        case module.GameStage.Play:
          if (this.props.isSelectionValid) {
            const text = this.props.isSelectionEmpty ? "pass" : "play";
            return (
              <div
                id="button"
                className="enabled"
                onClick={this.props.buttonHandler}
              >
                {text}
              </div>
            );
          } else {
            return <div id="button">play</div>;
          }
        case module.GameStage.Wait:
          return <div id="button">wait</div>;
        case module.GameStage.GameOver:
          switch (this.props.outcome) {
            case Outcome.Won:
              return (
                <div
                  id="button"
                  className="won"
                  onClick={this.props.buttonHandler}
                >
                  you won!
                </div>
              );
            case Outcome.Lost:
              return (
                <div
                  id="button"
                  className="lost"
                  onClick={this.props.buttonHandler}
                >
                  you lost.
                </div>
              );
            case Outcome.Tied:
              return (
                <div
                  id="button"
                  className="tied"
                  onClick={this.props.buttonHandler}
                >
                  you tied.
                </div>
              );
          }
      }
    }
  }

  type QRReaderProps = {
    outputQrObjectUrl: string | null;
    qrReadHandler: (imageData: ArrayBuffer) => void;
  };

  // Accept qr code images as input through click (file select dialog), drag,
  // or paste (with a button)
  class QRReader extends React.Component<QRReaderProps> {
    constructor(props: QRReaderProps) {
      super(props);

      this.dragOver = this.dragOver.bind(this);
      this.drop = this.drop.bind(this);
      this.onSelectFile = this.onSelectFile.bind(this);
      this.paste = this.paste.bind(this);
    }

    // Accept dragged files if they contain a file or a uri list
    dragOver(event: React.DragEvent) {
      if (
        event.dataTransfer.types.includes("text/uri-list") ||
        event.dataTransfer.items[0].kind == "file"
      ) {
        // Tell the broswer we want a copy-like effect so that it can provide
        // appropriate feedback
        event.dataTransfer.dropEffect = "copy";

        event.preventDefault();
      }
    }

    // Read qr code data from drag event
    drop(event: React.DragEvent) {
      event.preventDefault();

      if (event.dataTransfer.types.includes("text/uri-list")) {
        // Dragging from browser windows
        const uri = event.dataTransfer.getData("text/uri-list");

        // Warn against switching to the other player
        if (
          uri == this.props.outputQrObjectUrl &&
          !confirm("Switch to opponent's view?")
        ) {
          return;
        }

        // Feed the uri to an img element so that we can extract a png blob
        // and call qrReadHandler
        const img = new Image();
        // Allow reading images from other origins
        img.crossOrigin = "Anonymous";
        img.onload = () => {
          const canvas = document.createElement("canvas");
          canvas.width = img.naturalWidth;
          canvas.height = img.naturalHeight;
          canvas.getContext("2d")?.drawImage(img, 0, 0);

          canvas.toBlob((blob) => {
            blob?.arrayBuffer().then((arrayBuffer) => {
              this.props.qrReadHandler(arrayBuffer);
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
        createImageBitmap(file).then((bitmap) => {
          const canvas = document.createElement("canvas");
          canvas.width = bitmap.width;
          canvas.height = bitmap.height;
          canvas.getContext("2d")?.drawImage(bitmap, 0, 0);

          canvas.toBlob((blob) => {
            blob?.arrayBuffer().then((arrayBuffer) => {
              this.props.qrReadHandler(arrayBuffer);
            });
          });
        });
      }
    }

    // Read the selected file as a png blob and pass it to qrReadHandler
    onSelectFile(event: React.ChangeEvent) {
      const element = event.target as HTMLInputElement;

      const file = element.files && element.files[0];
      if (file) {
        createImageBitmap(file).then((bitmap) => {
          const canvas = document.createElement("canvas");
          canvas.width = bitmap.width;
          canvas.height = bitmap.height;
          canvas.getContext("2d")?.drawImage(bitmap, 0, 0);

          canvas.toBlob((blob) => {
            blob?.arrayBuffer().then((arrayBuffer) => {
              this.props.qrReadHandler(arrayBuffer);
            });
          });
        });
      }

      // clear the file from the input so that the input does not contain old
      // selections after those old selections have been processed
      element.value = "";
    }

    // Look for a png image in the user's clipboard and read it as a qr code.
    // This requires the user to grant a permission, and for now, non-textual
    // clipboard processing is only supported in Chrome
    paste() {
      try {
        navigator.permissions
          // clipboard-read is correct according to w3c, typescript thinks it's clipboard
          // @ts-ignore
          .query({ name: "clipboard-read" })
          .then((result) => {
            // If permission to read the clipboard is granted or if the user will
            // be prompted to allow it, we proceed.

            if (result.state == "granted" || result.state == "prompt") {
              // @ts-ignore
              navigator.clipboard.read().then((items) => {
                console.log(items);
                items[0].getType("image/png").then((blob: Blob | null) => {
                  if (blob) {
                    console.log(blob);
                    blob?.arrayBuffer().then((arrayBuffer: ArrayBuffer) => {
                      this.props.qrReadHandler(arrayBuffer);
                    });
                  }
                });
              });
            }
          });
      } catch (e) {
        console.error(e);
      }
    }

    render() {
      return (
        <>
          <label
            id="qr_reader"
            onDragOver={this.dragOver}
            onDragEnter={this.dragOver}
            onDrop={this.drop}
          >
            <input
              type="file"
              onChange={this.onSelectFile}
              style={{ display: "none" }}
            />
          </label>
          <div id="paste_button" onClick={this.paste}>
            {/* Paste svg icon */}
            <svg
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 24 24"
              width="24"
              height="24"
            >
              <path
                fillRule="evenodd"
                d="M5.962 2.513a.75.75 0 01-.475.949l-.816.272a.25.25 0 00-.171.237V21.25c0 .138.112.25.25.25h14.5a.25.25 0 00.25-.25V3.97a.25.25 0 00-.17-.236l-.817-.272a.75.75 0 01.474-1.424l.816.273A1.75 1.75 0 0121 3.97v17.28A1.75 1.75 0 0119.25 23H4.75A1.75 1.75 0 013 21.25V3.97a1.75 1.75 0 011.197-1.66l.816-.272a.75.75 0 01.949.475z"
              ></path>
              <path
                fillRule="evenodd"
                d="M7 1.75C7 .784 7.784 0 8.75 0h6.5C16.216 0 17 .784 17 1.75v1.5A1.75 1.75 0 0115.25 5h-6.5A1.75 1.75 0 017 3.25v-1.5zm1.75-.25a.25.25 0 00-.25.25v1.5c0 .138.112.25.25.25h6.5a.25.25 0 00.25-.25v-1.5a.25.25 0 00-.25-.25h-6.5z"
              ></path>
            </svg>
          </div>
        </>
      );
    }
  }

  type ScoresProps = {
    firstPlayerScore: number;
    secondPlayerScore: number;
  };

  // Display the players' scores
  class Scores extends React.Component<ScoresProps> {
    render() {
      return (
        <>
          <span id="firstPlayerScore">
            {this.props.firstPlayerScore}
            {this.props.firstPlayerScore == 1 ? "pt" : "pts"}
          </span>
          <span id="secondPlayerScore">
            {this.props.secondPlayerScore}
            {this.props.secondPlayerScore == 1 ? "pt" : "pts"}
          </span>
        </>
      );
    }
  }

  type HandSizesProps = {
    firstPlayerHandSize: number;
    secondPlayerHandSize: number;
  };

  // Display the players' hand sizes
  class HandSizes extends React.Component<HandSizesProps> {
    render() {
      return (
        <div id="hand_sizes">
          <span id="firstPlayerHandSize">{this.props.firstPlayerHandSize}</span>
          <span id="hand_separator">–</span>
          <span id="secondPlayerHandSize">
            {this.props.secondPlayerHandSize}
          </span>
        </div>
      );
    }
  }

  ReactDOM.render(<App />, document.body);
});
