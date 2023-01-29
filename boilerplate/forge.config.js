const config = require("./holssi.json");

module.exports = {
  packagerConfig: {
    asar: true,
    appBundleId: config.appId
  },
  rebuildConfig: {},
  makers: [
    {
      name: '@electron-forge/maker-squirrel',
      config: {},
    },
    {
      name: '@electron-forge/maker-zip',
      platforms: ['darwin'],
    },
  ],
};
