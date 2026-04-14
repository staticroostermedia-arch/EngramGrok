import * as vscode from 'vscode';
import { exec } from 'child_process';
import * as os from 'os';
import * as path from 'path';

let outputChannel: vscode.OutputChannel;

export function activate(context: vscode.ExtensionContext) {
    outputChannel = vscode.window.createOutputChannel("Engram");
    outputChannel.appendLine("Engram VSA Memory Engine Activated.");

    const engramBin = path.join(os.homedir(), '.local', 'bin', 'engram');

    let cmdRemember = vscode.commands.registerCommand('engram.rememberSelection', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showWarningMessage('No active editor to remember selection from.');
            return;
        }

        const selection = editor.document.getText(editor.selection);
        if (!selection) {
            vscode.window.showWarningMessage('Please highlight some text/code to remember.');
            return;
        }

        const concept = await vscode.window.showInputBox({ 
            prompt: 'Enter a conceptual identifier (e.g. auth_middleware_fix)' 
        });

        if (!concept) return;

        outputChannel.appendLine(`[REMEMBER] Storing concept: ${concept}...`);
        
        exec(`"${engramBin}" remember "${concept}" "${selection.replace(/"/g, '\\"')}"`, (error, stdout, stderr) => {
            if (error) {
                vscode.window.showErrorMessage(`Engram failed: ${stderr || error.message}`);
                return;
            }
            vscode.window.showInformationMessage(`Engram successfully remembered: ${concept}`);
            outputChannel.appendLine(stdout);
        });
    });

    let cmdRecall = vscode.commands.registerCommand('engram.recallContext', async () => {
        const query = await vscode.window.showInputBox({ 
            prompt: 'Query the geometric manifold (e.g. "how do we handle database migrations?")' 
        });

        if (!query) return;

        outputChannel.show();
        outputChannel.appendLine(`\n[RECALL] Scanning manifold for: "${query}"...`);
        
        exec(`"${engramBin}" recall "${query}" -k 5`, (error, stdout, stderr) => {
            if (error) {
                outputChannel.appendLine(`Error: ${stderr || error.message}`);
                vscode.window.showErrorMessage("Engram recall failed. See output channel for details.");
                return;
            }
            outputChannel.appendLine("=== K-NN Geometric Matches ===");
            outputChannel.appendLine(stdout);
        });
    });

    context.subscriptions.push(cmdRemember, cmdRecall);
}

export function deactivate() {
    if (outputChannel) {
        outputChannel.dispose();
    }
}
