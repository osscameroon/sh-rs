# Base structure
- The command line will be parsed and a list of linked token will be generated.
- Each command will be represented in a token.

## Token
- read: The stdin will be the output of the command given
- cmd: The command name
- params: The parameters of the command,
- status: The status of the execution of the command
- success: The next command to execute in case of success
- failure: The next command to execute in case of failure
- next: The next command to execute whatever the execution status

```sh
~/OSSCameroon/sh-rs> echo PNG Search Script; echo Search started && echo Searching... & ls | sort | head -2 | grep .png$ > result.txt || echo Not Found; echo Search End;
```
```rs   
[
    Token {
        cmd: Some(
            "echo",
        ),
        params: [
            "PNG",
            "Search",
            "Script",
        ],
        status: None,
        on_read: None,
        on_success: None,
        on_failure: None,
        next: None,
        stdin: None,
        stdout: None,
    },
    Token {
        cmd: Some(
            "echo",
        ),
        params: [
            "Not",
            "Found",
        ],
        status: None,
        on_read: None,
        on_success: None,
        on_failure: Some(
            Token {
                cmd: Some(
                    "grep",
                ),
                params: [
                    ".png$",
                ],
                status: None,
                on_read: Some(
                    Token {
                        cmd: Some(
                            "head",
                        ),
                        params: [
                            "-2",
                        ],
                        status: None,
                        on_read: Some(
                            Token {
                                cmd: Some(
                                    "sort",
                                ),
                                params: [],
                                status: None,
                                on_read: Some(
                                    Token {
                                        cmd: Some(
                                            "ls",
                                        ),
                                        params: [],
                                        status: None,
                                        on_read: None,
                                        on_success: None,
                                        on_failure: None,
                                        next: Some(
                                            Token {
                                                cmd: Some(
                                                    "echo",
                                                ),
                                                params: [
                                                    "Searching...",
                                                ],
                                                status: None,
                                                on_read: None,
                                                on_success: Some(
                                                    Token {
                                                        cmd: Some(
                                                            "echo",
                                                        ),
                                                        params: [
                                                            "Search",
                                                            "started",
                                                        ],
                                                        status: None,
                                                        on_read: None,
                                                        on_success: None,
                                                        on_failure: None,
                                                        next: None,
                                                        stdin: None,
                                                        stdout: None,
                                                    },
                                                ),
                                                on_failure: None,
                                                next: None,
                                                stdin: None,
                                                stdout: None,
                                            },
                                        ),
                                        stdin: None,
                                        stdout: None,
                                    },
                                ),
                                on_success: None,
                                on_failure: None,
                                next: None,
                                stdin: None,
                                stdout: None,
                            },
                        ),
                        on_success: None,
                        on_failure: None,
                        next: None,
                        stdin: None,
                        stdout: None,
                    },
                ),
                on_success: None,
                on_failure: None,
                next: None,
                stdin: Some(
                    "result.txt",
                ),
                stdout: None,
            },
        ),
        next: None,
        stdin: None,
        stdout: None,
    },
    Token {
        cmd: Some(
            "echo",
        ),
        params: [
            "Search",
            "End",
        ],
        status: None,
        on_read: None,
        on_success: None,
        on_failure: None,
        next: None,
        stdin: None,
        stdout: None,
    },
    Token {
        cmd: None,
        params: [],
        status: None,
        on_read: None,
        on_success: None,
        on_failure: None,
        next: None,
        stdin: None,
        stdout: None,
    },
]
```