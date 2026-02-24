import * as vscode from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext): void {
  const config = vscode.workspace.getConfiguration("truss");
  const enabled = config.get<boolean>("enable", true);

  if (!enabled) {
    return;
  }

  const lspPath = config.get<string>("lspPath", "truss-lsp");

  const serverOptions: ServerOptions = {
    command: lspPath,
    transport: TransportKind.stdio,
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      {
        scheme: "file",
        language: "yaml",
        pattern: "**/.github/workflows/*.{yml,yaml}",
      },
    ],
    synchronize: {
      fileEvents:
        vscode.workspace.createFileSystemWatcher(
          "**/.github/workflows/*.{yml,yaml}"
        ),
    },
  };

  client = new LanguageClient(
    "truss",
    "Truss Language Server",
    serverOptions,
    clientOptions
  );

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
