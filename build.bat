REM Install wasm-pack from here https://rustwasm.github.io/wasm-pack/installer/
wasm-pack build --target web || goto :error

mkdir deploy
copy pkg\* deploy || goto :error
copy static\* deploy || goto :error

REM Build OK. Now run with:
REM     microserver --port 8000 --no-spa deploy

goto :EOF

:error
echo Failed with error #%errorlevel%.
exit /b %errorlevel%
