// Single re-export surface for the generated protobuf types: route components import
// from here ($lib/pb), never the deep generated paths (those are build artifacts).
export type {
	Category,
	ChapterInfo,
	MangaSummary
} from '../gen/koma/v1/koma_pb';
