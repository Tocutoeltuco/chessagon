export class AssetManager {
  constructor() {
    this.assets = {};
    this.loading = {};
    this.lastId = 0;
    this.waiting = [];
  }

  _markLoading() {
    const id = this.lastId++;
    this.loading[id] = true;
    return id;
  }
  _markDone(id) {
    delete this.loading[id];
    this._maybeReady();
  }

  /**
   * Gets an asset that was already loaded.
   * @param {string} name
   * @returns {Image | undefined} asset
   */
  get(name) {
    return this.assets[name];
  }

  /**
   * Internal method. Fetches an image without marking any load state.
   * @param {string} url
   * @returns {Promise<string>} content
   */
  _fetch(url) {
    return new Promise((resolve, reject) => {
      return fetch(url)
        .then((response) => response.text())
        .then(resolve)
        .catch(reject);
    });
  }

  /**
   * Fetches an asset. Does not save nor create an Image
   * @param {string} url
   * @returns {Promise<string>} content
   */
  fetch(url) {
    const id = this._markLoading();

    return new Promise((resolve, reject) => {
      this._fetch(url)
        .then((content) => {
          this._markDone(id);
          resolve(content);
        })
        .catch((err) => {
          this._markDone(id);
          reject(err);
        });
    });
  }

  /**
   * Fetches an asset. Creates an Image and then saves it.
   * @param {string} url
   * @returns {Promise<Image>} content
   */
  fetchAndSave(url) {
    const id = this._markLoading();
    return new Promise((resolve, reject) => {
      this._fetch(url)
        .then((content) => this.create(url, content))
        .then((img) => {
          this._markDone(id);
          resolve(img);
        })
        .catch((err) => {
          this._markDone(id);
          reject(err);
        });
    });
  }

  /**
   * Creates an asset on the fly and saves it.
   * @param {string} name Name to use on AssetManager.get
   * @param {string} content
   * @returns {Promise<[Image, string]>}
   */
  create(name, content) {
    const img = new Image();
    const id = this._markLoading();

    const promise = new Promise((resolve, _) => {
      img.onload = () => {
        this.assets[name] = img;
        resolve(img);
        this._markDone(id);
      };
      img.src = "data:image/svg+xml;base64," + btoa(content);
    });
    return promise;
  }

  get isReady() {
    // If there are any items in waiting list, we're not ready
    for (const _ of Object.entries(this.loading)) {
      return false;
    }

    return true;
  }

  _maybeReady() {
    if (!this.isReady) return;

    while (this.waiting.length > 0) {
      const resolve = this.waiting.pop();
      resolve();
    }
  }

  /**
   * Waits until all assets are loaded.
   * @returns {Promise<undefined>}
   */
  waitReady() {
    return new Promise((resolve, _) => {
      if (this.isReady) return resolve();

      this.waiting.push(resolve);
    });
  }
}
