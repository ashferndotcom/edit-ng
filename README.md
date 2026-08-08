# edit-ng

edit-ng is a text editor based on [microsoft/edit](https://www.github.com/microsoft/edit).

## Installation
Install the latest version of edit-ng from [GitHub Releases](https://github.com/ashferndotcom/edit-ng/releases/tag/v0.1.0) and `chmod +x ~/Downloads/edit-ng` to try it before manually installing or run the command below to make it more convenient:
```bash
curl -fsSL https://github.com/ashferndotcom/edit-ng/releases/latest/download/edit-ng -o edit-ng && chmod +x edit-ng && ./edit-ng
```
Or, if you want to make the editor be runnable as a command in your PATH and then run it:

```bash
curl -fsSL https://github.com/ashferndotcom/edit-ng/releases/latest/download/edit-ng -o edit-ng && chmod +x edit-ng && sudo mkdir -p /opt/edit-ng && sudo mv edit-ng /opt/edit-ng/ && sudo ln -s /opt/edit-ng/edit-ng /usr/local/bin/edit-ng && edit-ng
```

## Usage
Now, open the executable in the terminal by executing:

```bash
cd ~/Downloads
./edit-ng
```
Or, if you used the second method (PATH Method):

```bash
edit-ng
```
This will open edit-ng for the first time or run it again. 

## Contributing

Pull requests are welcome. For major changes, please open an issue first
to discuss what you would like to change.

## License

[MIT](https://choosealicense.com/licenses/mit/)
