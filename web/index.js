// polyfills
import 'js-polyfills/keyboard';
import 'wcwidth';

import $ from 'jquery';
import 'jquery.terminal';
import 'jquery.terminal/js/unix_formatting';

import 'jquery.terminal/css/jquery.terminal.min.css';
import './style.css';

function setColorscheme() {
    // HACK: There is no official way to set colorschemes, so we resort to overwriting it
    // $.terminal.ansi_colors is in unix_formatting.js

    $.terminal.ansi_colors.normal = {
        // One Half Dark
        "black": "#282c34",
        "red": "#e06c75",
        "green": "#98c379",
        "yellow": "#e5c07b",
        "blue": "#61afef",
        "magenta": "#c678dd",
        "cyan": "#56b6c2",
        "white": "#dcdfe4"
    };
}

import('./pkg').then(({ version, Repl, ResponseKind }) => {
    const repl = new Repl();

    $(document).ready(() => {
        $("#terminal").terminal(
            function (input) {
                setColorscheme();

                const response = repl.run(input);
                switch (response.kind) {
                    case ResponseKind.Clear:
                        this.clear();
                        break;
                    case ResponseKind.Reset:
                        this.reset();
                        break;
                }

                return response.message;
            }, {
            clear: false,
            completion: function (_, cb) {
                cb(repl.completion_candidates().split('\n'))
            },
            greetings: `Welcome to [[b;#a6e22e;]beek] ${version()}`,
        }
        );
    });
}).catch(console.error);
