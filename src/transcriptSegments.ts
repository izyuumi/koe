export interface TranscriptSegment {
  text: string;
  start_ms: number;
  end_ms: number;
}

function getSharedPrefixLength(a: string, b: string): number {
  const maxLength = Math.min(a.length, b.length);
  let index = 0;

  while (index < maxLength && a[index] === b[index]) {
    index += 1;
  }

  return index;
}

export function getTranscriptText(segments: TranscriptSegment[]): string {
  return segments.map((segment) => segment.text).join("");
}

export function reconcileTranscriptSegments(
  segments: TranscriptSegment[],
  nextTranscript: string,
  endMs: number,
): TranscriptSegment[] {
  const previousTranscript = getTranscriptText(segments);
  if (previousTranscript === nextTranscript) {
    return segments;
  }

  const sharedPrefixLength = getSharedPrefixLength(previousTranscript, nextTranscript);
  const nextSegments: TranscriptSegment[] = [];
  let remainingPrefix = sharedPrefixLength;

  for (const segment of segments) {
    if (remainingPrefix <= 0) {
      break;
    }

    if (remainingPrefix >= segment.text.length) {
      nextSegments.push(segment);
      remainingPrefix -= segment.text.length;
      continue;
    }

    nextSegments.push({
      ...segment,
      text: segment.text.slice(0, remainingPrefix),
    });
    remainingPrefix = 0;
    break;
  }

  const revisedText = nextTranscript.slice(sharedPrefixLength);
  if (revisedText) {
    const revisedStartMs =
      nextSegments.length > 0 ? nextSegments[nextSegments.length - 1].end_ms : 0;
    nextSegments.push({
      text: revisedText,
      start_ms: revisedStartMs,
      end_ms: Math.max(endMs, revisedStartMs),
    });
  }

  return nextSegments;
}
