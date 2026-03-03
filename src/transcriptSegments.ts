export interface TranscriptSegment {
  text: string;
  start_ms: number;
  end_ms: number;
}

const INSIGNIFICANT_TRANSCRIPT_CHARS = /[\s\p{P}]/u;

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

function getSemanticCharCount(text: string): number {
  let count = 0;

  for (const char of text) {
    if (!INSIGNIFICANT_TRANSCRIPT_CHARS.test(char)) {
      count += 1;
    }
  }

  return count;
}

function getComparableTranscript(text: string): string {
  let comparable = "";

  for (const char of text) {
    if (!INSIGNIFICANT_TRANSCRIPT_CHARS.test(char)) {
      comparable += char.toLocaleLowerCase();
    }
  }

  return comparable;
}

function splitTranscriptAcrossSegments(
  segments: TranscriptSegment[],
  nextTranscript: string,
): TranscriptSegment[] {
  const semanticBoundaries = segments.map((segment) => getSemanticCharCount(segment.text));
  let nextIndex = 0;
  let consumedSemanticChars = 0;
  let targetSemanticChars = 0;

  return segments.map((segment, segmentIndex) => {
    targetSemanticChars += semanticBoundaries[segmentIndex];
    const sliceStart = nextIndex;

    if (segmentIndex === segments.length - 1) {
      nextIndex = nextTranscript.length;
    } else {
      while (
        nextIndex < nextTranscript.length &&
        consumedSemanticChars < targetSemanticChars
      ) {
        const char = nextTranscript[nextIndex];
        if (!INSIGNIFICANT_TRANSCRIPT_CHARS.test(char)) {
          consumedSemanticChars += 1;
        }
        nextIndex += 1;
      }

      while (
        nextIndex < nextTranscript.length &&
        INSIGNIFICANT_TRANSCRIPT_CHARS.test(nextTranscript[nextIndex])
      ) {
        nextIndex += 1;
      }
    }

    return {
      ...segment,
      text: nextTranscript.slice(sliceStart, nextIndex),
    };
  });
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

  if (
    segments.length > 0 &&
    getComparableTranscript(previousTranscript) === getComparableTranscript(nextTranscript)
  ) {
    return splitTranscriptAcrossSegments(segments, nextTranscript);
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

    const ratio = remainingPrefix / segment.text.length;
    const adjustedEndMs = segment.start_ms + Math.round((segment.end_ms - segment.start_ms) * ratio);
    nextSegments.push({
      ...segment,
      text: segment.text.slice(0, remainingPrefix),
      end_ms: adjustedEndMs,
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
