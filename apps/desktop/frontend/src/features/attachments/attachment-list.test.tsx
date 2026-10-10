import { render, screen } from "@testing-library/react";
import type { Attachment } from "@jabex/types";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { AttachmentList } from "./attachment-list";

vi.mock("@/hooks/use-locale", () => ({
  useTranslations: () => ({
    attachments: {
      emptyTitle: "No attachments yet",
      emptyHint: "Add media",
      remove: "Remove attachment",
      types: {
        image: "Image",
        audio: "Audio",
        video: "Video",
        pdf: "PDF",
      },
    },
  }),
}));

const sample: Attachment = {
  id: "11111111-1111-7111-8111-111111111111",
  ownerType: "card",
  ownerId: "22222222-2222-7222-8222-222222222222",
  fileName: "diagram.png",
  mimeType: "image/png",
  sizeBytes: 2048,
  hash: "abc",
  relativePath: "media/attachments/11111111-1111-7111-8111-111111111111/original.png",
  revision: 1,
  createdAt: "2026-01-01T00:00:00Z",
  updatedAt: "2026-01-01T00:00:00Z",
  deletedAt: null,
};

describe("AttachmentList", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders empty state", () => {
    render(<AttachmentList attachments={[]} />);
    expect(screen.getByText("No attachments yet")).toBeInTheDocument();
  });

  it("renders attachment rows", () => {
    render(<AttachmentList attachments={[sample]} />);
    expect(screen.getByText("diagram.png")).toBeInTheDocument();
    expect(screen.getByText(/Image/)).toBeInTheDocument();
  });
});
