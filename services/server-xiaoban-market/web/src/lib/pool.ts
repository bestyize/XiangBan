/** 按顺序取任务、最多 `limit` 路并发执行；`stopped()` 为真后不再取新任务（已在跑的会跑完）。 */
export async function runPool<T>(items: T[], limit: number, task: (item: T) => Promise<void>, stopped: () => boolean = () => false): Promise<void> {
  const queue = [...items];
  const worker = async () => {
    while (queue.length && !stopped()) await task(queue.shift() as T);
  };
  await Promise.all(Array.from({ length: Math.max(1, Math.min(limit, items.length)) }, worker));
}
