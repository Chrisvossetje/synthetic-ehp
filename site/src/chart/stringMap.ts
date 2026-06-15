interface Stringifiable {
    toString(): string;
}

export class ToStringMap<K extends Stringifiable, V> {
    map: {[key: string]: V};
    constructor() {
        this.map = {};
    }
    public clear() {
        this.map = {};
    }
    public set(key: K, value: V) {
        this.map[key.toString()] = value;
    }
    public get(key: K): V | undefined {
        return this.map[key.toString()];
    }
    public has(key: K): boolean {
        if (this.map[key.toString()] == undefined) {
            return false;
        } else {
            return true;
        }
    }
}
