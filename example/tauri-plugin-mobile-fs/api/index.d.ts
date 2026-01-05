export function ping(value: string): Promise<string | null>

export function getFileDescriptor(uri: string, mode: string): Promise<number | null>
export function getFileName(uri: string): Promise<string | null>
