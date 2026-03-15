# Stone Tauri - Improvement Roadmap

> Generated: 2026-03-15
> Status: In Progress

## 🔴 Phase 1: Critical Fixes (MUST FIX) ✅ COMPLETED

### TypeScript Compilation Errors
- [x] **TS-1**: Fix BlockDragDrop.ts line 74 - condition always true
- [x] **TS-2**: Fix BlockDragDrop.ts line 84 - remove unused `view` parameter
- [x] **TS-3**: Fix BlockDragDrop.ts line 177 - fix `near` property type error
- [x] **TS-4**: Fix MarkdownPaste.ts line 23 - remove unused `slice` parameter
- [x] **TS-5**: Fix TableNavigation.ts line 53 - remove unused `cellDepth` variable
- [x] **TS-6**: Fix TableNavigation.ts line 201 - remove unused `state` variable

### Critical getState() Violations
- [x] **GS-1**: Fix useMLStatus.ts (lines 21, 43, 59) - use selectors ✅
- [x] **GS-2**: Fix useNoteAPI.ts toggles (lines 196, 218, 238) - pass note or use selectors ✅
- [ ] **GS-3**: Fix useJournalActions.ts (lines 54, 88, 130) - use selectors (lower priority)
- [ ] **GS-4**: Fix useCommandCenter.tsx (lines 60, 66, 82, 95, 139, 151) - use selectors (lower priority)
- [ ] **GS-5**: Fix useDocumentBuffer.ts (lines 185, 204) - use selectors (lower priority)
- [ ] **GS-6**: Fix FileLeaf.tsx line 57 - use selector hook (lower priority)

## 🟠 Phase 2: Performance Improvements (HIGH PRIORITY)

### React.memo on List Components
- [ ] **PERF-1**: Add React.memo to NoteListFileItem.tsx
- [ ] **PERF-2**: Add React.memo to NoteListFolderItem.tsx
- [ ] **PERF-3**: Add React.memo to CommandItemRow.tsx
- [ ] **PERF-4**: Add React.memo to TaskSection.tsx
- [ ] **PERF-5**: Add React.memo to TreeItem.tsx
- [ ] **PERF-6**: Add React.memo to ListItem.tsx
- [ ] **PERF-7**: Add React.memo to CompactCard.tsx

### useMemo/useCallback Optimizations
- [ ] **PERF-8**: Audit components with array operations for useMemo
- [ ] **PERF-9**: Audit event handlers for useCallback in list items

## 🟡 Phase 3: Code Quality (MEDIUM PRIORITY)

### Extract Magic Numbers
- [ ] **QUAL-1**: Extract cache TTL constants to config
- [ ] **QUAL-2**: Extract debounce delays to config
- [ ] **QUAL-3**: Extract other magic numbers

### Standardize Error Handling
- [ ] **QUAL-4**: Define error handling pattern
- [ ] **QUAL-5**: Implement consistent error handling in hooks
- [ ] **QUAL-6**: Implement consistent error handling in components

### Refactor Complex Functions
- [ ] **QUAL-7**: Split useDocumentBuffer.ts into smaller hooks
- [ ] **QUAL-8**: Review other hooks >200 lines for splitting

### Clean Up Technical Debt
- [ ] **QUAL-9**: Remove commented code in useFileTreeAPI.ts
- [ ] **QUAL-10**: Address TODO comments for incomplete features
- [ ] **QUAL-11**: Review and fix improper dependency arrays

## 🟢 Phase 4: Testing (MEDIUM PRIORITY)

### Test Infrastructure
- [ ] **TEST-1**: Set up Jest and React Testing Library
- [ ] **TEST-2**: Configure test environment

### Store Tests
- [ ] **TEST-3**: Write tests for noteStore
- [ ] **TEST-4**: Write tests for fileTreeStore
- [ ] **TEST-5**: Write tests for documentBufferStore
- [ ] **TEST-6**: Write tests for other stores

### Hook Tests
- [ ] **TEST-7**: Write tests for useNoteAPI
- [ ] **TEST-8**: Write tests for useFileTreeAPI
- [ ] **TEST-9**: Write tests for useNoteCache
- [ ] **TEST-10**: Write tests for useAutosave

### Utility Tests
- [ ] **TEST-11**: Write tests for markdown parser
- [ ] **TEST-12**: Write tests for markdown serializer
- [ ] **TEST-13**: Write tests for path utilities

## 📚 Phase 5: Documentation (LOW PRIORITY)

### JSDoc Coverage
- [ ] **DOC-1**: Add JSDoc to remaining 26% of files
- [ ] **DOC-2**: Document interface properties in type definitions
- [ ] **DOC-3**: Document hook parameters and return types

### Type Improvements
- [ ] **DOC-4**: Replace `any` types with proper types
- [ ] **DOC-5**: Add stricter type definitions

## Progress Summary

**Total Tasks**: 52
**Completed**: 8
**In Progress**: 0
**Blocked**: 0

### Phase Progress
- 🔴 Phase 1 (Critical): 8/12 (67%) - TypeScript errors ✅ | Critical getState() ✅
- 🟠 Phase 2 (Performance): 0/9 (0%)
- 🟡 Phase 3 (Quality): 0/8 (0%)
- 🟢 Phase 4 (Testing): 0/11 (0%)
- 📚 Phase 5 (Documentation): 0/5 (0%)

### Recent Accomplishments (2026-03-15)
1. ✅ Fixed all 6 TypeScript compilation errors
2. ✅ Fixed critical getState() violations in useMLStatus and useNoteAPI
3. ✅ Build now passes without errors
4. ✅ ML status hooks now properly reactive
5. ✅ Note toggle operations use proper selectors

---

## Notes

- Focus on Phase 1 first (blocks production builds)
- Phase 2 has high user-facing impact
- Phases 3-5 can be done incrementally
- Update checkboxes as tasks are completed
