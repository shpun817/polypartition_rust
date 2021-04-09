export async function readFile(pathToFile: string): Promise<string> {
    const object = await fetch(pathToFile);
    const text = await object.text();
    return text;
}