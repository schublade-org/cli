#!/usr/bin/env node
"use strict";

const { ensureBinary } = require("./lib/download");

ensureBinary()
  .then((binary) => {
    console.log(`schublade: installed ${binary}`);
  })
  .catch((error) => {
    console.error(`schublade install failed: ${error.message}`);
    process.exit(1);
  });
