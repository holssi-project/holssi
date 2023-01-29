module.exports = {
  packagerConfig: {
    asar: true,
    appBundleId: "dev.jedeop.holssi.example"
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
