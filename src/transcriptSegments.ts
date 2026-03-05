export interface TranscriptSegment {
  text: string;
  start_ms: number;
  end_ms: number;
}

const INSIGNIFICANT_TRANSCRIPT_CHARS = /[\s\p{P}]/u;

function getSharedPrefixLength(a: string, b: string): number {
  const aCodePoints = Array.from(a);
  const bCodePoints = Array.from(b);
  const maxLength = Math.min(aCodePoints.length, bCodePoints.length);
  let index = 0;

  while (index < maxLength && aCodePoints[index] === bCodePoints[index]) {
    index += 1;
  }

  return index;
}

function getCodePointLength(text: string): number {
  return Array.from(text).length;
}

function sliceByCodePoints(text: string, start: number, end?: number): string {
  return Array.from(text).slice(start, end).join("");
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
  const nextCodePoints = Array.from(nextTranscript);
  let nextIndex = 0;
  let consumedSemanticChars = 0;
  let targetSemanticChars = 0;

  const splitSegments = segments.map((segment, segmentIndex) => {
    targetSemanticChars += semanticBoundaries[segmentIndex];
    const sliceStart = nextIndex;

    if (segmentIndex === segments.length - 1) {
      nextIndex = nextCodePoints.length;
    } else {
      while (
        nextIndex < nextCodePoints.length &&
        consumedSemanticChars < targetSemanticChars
      ) {
        const char = nextCodePoints[nextIndex];
        if (!INSIGNIFICANT_TRANSCRIPT_CHARS.test(char)) {
          consumedSemanticChars += 1;
        }
        nextIndex += 1;
      }

      while (
        nextIndex < nextCodePoints.length &&
        INSIGNIFICANT_TRANSCRIPT_CHARS.test(nextCodePoints[nextIndex])
      ) {
        nextIndex += 1;
      }
    }

    return {
      ...segment,
      text: nextCodePoints.slice(sliceStart, nextIndex).join(""),
    };
  });

  return mergeInsignificantSegments(splitSegments);
}

function isInsignificantSegmentText(text: string): boolean {
  return getSemanticCharCount(text) === 0;
}

function mergeInsignificantSegments(segments: TranscriptSegment[]): TranscriptSegment[] {
  const normalized: TranscriptSegment[] = [];
  let leadingInsignificantText = "";
  let leadingStartMs: number | null = null;

  for (const segment of segments) {
    if (!isInsignificantSegmentText(segment.text)) {
      if (leadingInsignificantText) {
        normalized.push({
          ...segment,
          text: `${leadingInsignificantText}${segment.text}`,
          start_ms: leadingStartMs ?? segment.start_ms,
        });
        leadingInsignificantText = "";
        leadingStartMs = null;
      } else {
        normalized.push({ ...segment });
      }
      continue;
    }

    const previousSegment = normalized[normalized.length - 1];
    if (previousSegment) {
      previousSegment.text += segment.text;
      previousSegment.end_ms = segment.end_ms;
      continue;
    }

    if (leadingStartMs === null) {
      leadingStartMs = segment.start_ms;
    }
    leadingInsignificantText += segment.text;
  }

  return normalized;
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

    const segmentLength = getCodePointLength(segment.text);
    if (remainingPrefix >= segmentLength) {
      nextSegments.push(segment);
      remainingPrefix -= segmentLength;
      continue;
    }

    const ratio = remainingPrefix / segmentLength;
    const adjustedEndMs = segment.start_ms + Math.round((segment.end_ms - segment.start_ms) * ratio);
    nextSegments.push({
      ...segment,
      text: sliceByCodePoints(segment.text, 0, remainingPrefix),
      end_ms: adjustedEndMs,
    });
    remainingPrefix = 0;
    break;
  }

  const revisedText = sliceByCodePoints(nextTranscript, sharedPrefixLength);
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
