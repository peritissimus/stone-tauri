#!/usr/bin/env node

/**
 * Comprehensive version tagging script using semver library
 * Supports all standard version bump types and custom versions
 * Integrates with git for tag creation and management
 *
 * Syncs version across:
 *   - package.json
 *   - src-tauri/Cargo.toml
 *   - src-tauri/tauri.conf.json
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const semver = require('semver');

// Color codes for terminal output
const colors = {
  reset: '\x1b[0m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  magenta: '\x1b[35m',
  cyan: '\x1b[36m',
  white: '\x1b[37m',
};

// Helper functions for colored output
const log = {
  info: (msg) => console.log(`${colors.blue}\u2139${colors.reset}  ${msg}`),
  success: (msg) => console.log(`${colors.green}\u2713${colors.reset}  ${msg}`),
  warning: (msg) => console.log(`${colors.yellow}\u26A0${colors.reset}  ${msg}`),
  error: (msg) => console.log(`${colors.red}\u2717${colors.reset}  ${msg}`),
  header: (msg) => console.log(`\n${colors.cyan}\u25B6${colors.reset}  ${colors.white}${msg}${colors.reset}`),
};

// Parse command line arguments
function parseArgs() {
  const args = process.argv.slice(2);
  const options = {
    bumpType: null,
    preid: 'rc',
    dryRun: false,
    noGit: false,
    push: false,
    force: false,
    message: null,
    annotate: true,
    sign: false,
    verbose: false,
  };

  const validBumpTypes = [
    'major',
    'minor',
    'patch',
    'premajor',
    'preminor',
    'prepatch',
    'prerelease',
    'release',
  ];

  let i = 0;
  while (i < args.length) {
    const arg = args[i];

    if (arg === '--preid') {
      options.preid = args[++i];
    } else if (arg === '--dry-run' || arg === '-d') {
      options.dryRun = true;
    } else if (arg === '--no-git') {
      options.noGit = true;
    } else if (arg === '--push') {
      options.push = true;
    } else if (arg === '--force' || arg === '-f') {
      options.force = true;
    } else if (arg === '--message' || arg === '-m') {
      options.message = args[++i];
    } else if (arg === '--no-annotate') {
      options.annotate = false;
    } else if (arg === '--sign' || arg === '-s') {
      options.sign = true;
    } else if (arg === '--verbose' || arg === '-v') {
      options.verbose = true;
    } else if (arg === '--help' || arg === '-h') {
      printHelp();
      process.exit(0);
    } else if (arg.startsWith('-')) {
      log.error(`Unknown option: ${arg}`);
      printHelp();
      process.exit(1);
    } else {
      if (!options.bumpType) {
        if (validBumpTypes.includes(arg)) {
          options.bumpType = arg;
        } else if (semver.valid(arg)) {
          options.bumpType = 'custom';
          options.customVersion = arg;
        } else {
          log.error(`Invalid version bump type or version: ${arg}`);
          printHelp();
          process.exit(1);
        }
      }
    }
    i++;
  }

  if (!options.bumpType) {
    options.bumpType = 'patch';
  }

  return options;
}

// Print help message
function printHelp() {
  console.log(`
${colors.cyan}Stone Version Tagging Script${colors.reset}

Syncs version across package.json, src-tauri/Cargo.toml, and src-tauri/tauri.conf.json.

${colors.white}USAGE:${colors.reset}
  pnpm tag [bump-type|version] [options]

${colors.white}BUMP TYPES:${colors.reset}
  major         Bump major version (1.0.0 -> 2.0.0)
  minor         Bump minor version (1.0.0 -> 1.1.0)
  patch         Bump patch version (1.0.0 -> 1.0.1) [default]
  premajor      Pre-release major (1.0.0 -> 2.0.0-rc.0)
  preminor      Pre-release minor (1.0.0 -> 1.1.0-rc.0)
  prepatch      Pre-release patch (1.0.0 -> 1.0.1-rc.0)
  prerelease    Increment pre-release (1.0.0-rc.0 -> 1.0.0-rc.1)
  release       Graduate pre-release (1.0.0-rc.0 -> 1.0.0)
  <version>     Set specific version (e.g., 2.3.4-beta.1)

${colors.white}OPTIONS:${colors.reset}
  --preid <id>        Prerelease identifier (default: rc)
  --dry-run, -d       Preview changes without applying
  --no-git            Skip git operations
  --push              Push changes and tags to remote (default: false)
  --force, -f         Force operations (skip safety checks)
  --message, -m       Custom tag message
  --no-annotate       Create lightweight tag instead of annotated
  --sign, -s          Sign tag with GPG
  --verbose, -v       Verbose output
  --help, -h          Show this help message

${colors.white}EXAMPLES:${colors.reset}
  pnpm tag patch                          # Bump patch version
  pnpm tag patch --push                   # Bump patch and push to remote
  pnpm tag minor --dry-run                # Preview minor bump
  pnpm tag premajor --preid beta          # Start beta for major version
  pnpm tag 2.0.0                          # Set specific version
  pnpm tag prerelease --push              # Increment prerelease and push
  pnpm tag major -m "Breaking changes" --sign --push
  `);
}

// Execute command and return output
function exec(cmd, options = {}) {
  try {
    const output = execSync(cmd, {
      encoding: 'utf8',
      stdio: options.silent ? 'pipe' : 'inherit',
      ...options,
    });
    return output ? output.trim() : '';
  } catch (error) {
    if (options.ignoreError) {
      return '';
    }
    throw error;
  }
}

// Get current git branch
function getCurrentBranch() {
  return exec('git rev-parse --abbrev-ref HEAD', { silent: true });
}

// Check if working directory is clean
function isWorkingDirectoryClean() {
  const status = exec('git status --porcelain', { silent: true });
  return status.length === 0;
}

// Check if tag exists
function tagExists(tag) {
  const result = exec(`git tag -l "${tag}"`, { silent: true });
  return result.length > 0;
}

// Resolve path relative to project root
function projectPath(relativePath) {
  return path.resolve(__dirname, '..', relativePath);
}

// --- File readers/writers for each version source ---

function readPackageJson() {
  const filePath = projectPath('package.json');
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function writePackageJson(data) {
  const filePath = projectPath('package.json');
  fs.writeFileSync(filePath, JSON.stringify(data, null, 2) + '\n');
}

function readTauriConf() {
  const filePath = projectPath('src-tauri/tauri.conf.json');
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function writeTauriConf(data) {
  const filePath = projectPath('src-tauri/tauri.conf.json');
  fs.writeFileSync(filePath, JSON.stringify(data, null, 2) + '\n');
}

function readCargoVersion() {
  const filePath = projectPath('src-tauri/Cargo.toml');
  const content = fs.readFileSync(filePath, 'utf8');
  const match = content.match(/^version\s*=\s*"([^"]+)"/m);
  return match ? match[1] : null;
}

function writeCargoVersion(newVersion) {
  const filePath = projectPath('src-tauri/Cargo.toml');
  let content = fs.readFileSync(filePath, 'utf8');
  content = content.replace(
    /^(version\s*=\s*)"[^"]+"/m,
    `$1"${newVersion}"`
  );
  fs.writeFileSync(filePath, content);
}

// --- Backup / restore ---

function createBackups() {
  const files = [
    'package.json',
    'src-tauri/Cargo.toml',
    'src-tauri/tauri.conf.json',
  ];
  const backups = [];
  for (const file of files) {
    const full = projectPath(file);
    const backup = full + '.backup';
    fs.copyFileSync(full, backup);
    backups.push(backup);
  }
  return backups;
}

function restoreBackups(backups) {
  for (const backup of backups) {
    const original = backup.replace('.backup', '');
    if (fs.existsSync(backup)) {
      fs.copyFileSync(backup, original);
      fs.unlinkSync(backup);
    }
  }
}

function cleanupBackups(backups) {
  for (const backup of backups) {
    if (fs.existsSync(backup)) {
      fs.unlinkSync(backup);
    }
  }
}

// Calculate new version
function calculateNewVersion(currentVersion, bumpType, options) {
  if (bumpType === 'custom') {
    return options.customVersion;
  }

  if (bumpType === 'release') {
    const version = semver.parse(currentVersion);
    if (version && version.prerelease.length > 0) {
      return `${version.major}.${version.minor}.${version.patch}`;
    }
    return currentVersion;
  }

  const newVersion = semver.inc(currentVersion, bumpType, options.preid);
  if (!newVersion) {
    throw new Error(`Failed to calculate new version from ${currentVersion} with bump type ${bumpType}`);
  }
  return newVersion;
}

// Update all version files, returns list of updates for summary
function updateAllVersions(newVersion, options) {
  const backups = createBackups();

  try {
    const updates = [];

    // 1. package.json
    const pkg = readPackageJson();
    const oldVersion = pkg.version;
    pkg.version = newVersion;
    if (!options.dryRun) {
      writePackageJson(pkg);
    }
    updates.push({ file: 'package.json', oldVersion, newVersion });

    // 2. src-tauri/tauri.conf.json
    const tauriConf = readTauriConf();
    const oldTauriVersion = tauriConf.version;
    tauriConf.version = newVersion;
    if (!options.dryRun) {
      writeTauriConf(tauriConf);
    }
    updates.push({ file: 'src-tauri/tauri.conf.json', oldVersion: oldTauriVersion, newVersion });

    // 3. src-tauri/Cargo.toml
    const oldCargoVersion = readCargoVersion();
    if (!options.dryRun) {
      writeCargoVersion(newVersion);
    }
    updates.push({ file: 'src-tauri/Cargo.toml', oldVersion: oldCargoVersion, newVersion });

    if (!options.dryRun) {
      cleanupBackups(backups);
    } else {
      cleanupBackups(backups);
    }

    return updates;
  } catch (error) {
    if (!options.dryRun) {
      log.error('Error updating versions, restoring from backups...');
      restoreBackups(backups);
    } else {
      cleanupBackups(backups);
    }
    throw error;
  }
}

// Create git tag
function createGitTag(version, options) {
  const tag = `v${version}`;

  let tagCmd = 'git tag';

  if (options.annotate) {
    const message = options.message || `Release version ${version}`;
    tagCmd += ` -a "${tag}" -m "${message}"`;

    if (options.sign) {
      tagCmd += ' -s';
    }
  } else {
    tagCmd += ` "${tag}"`;
  }

  if (!options.dryRun) {
    exec(tagCmd);
  }

  return tag;
}

// Commit version bump changes
function commitChanges(version, updates, options) {
  if (options.dryRun) {
    return;
  }

  for (const update of updates) {
    exec(`git add "${update.file}"`);
  }

  const fileList = updates.map(u => `  - ${u.file}: ${u.oldVersion} -> ${u.newVersion}`).join('\n');
  const commitMessage = `chore: bump version to ${version}\n\nUpdated files:\n${fileList}`;

  exec(`git commit -m "${commitMessage}"`);
}

// Push changes and tags to remote
function pushChanges(tag, options) {
  if (options.dryRun || !options.push) {
    return;
  }

  const currentBranch = getCurrentBranch();

  log.info(`Pushing changes to ${currentBranch}...`);
  exec(`git push origin ${currentBranch}`);

  log.info(`Pushing tag ${tag}...`);
  exec(`git push origin ${tag}`);
}

// Verify all version files are in sync
function checkVersionSync() {
  const pkgVersion = readPackageJson().version;
  const tauriVersion = readTauriConf().version;
  const cargoVersion = readCargoVersion();

  if (pkgVersion !== tauriVersion || pkgVersion !== cargoVersion) {
    log.warning('Version files are out of sync:');
    log.info(`  package.json:            ${pkgVersion}`);
    log.info(`  src-tauri/tauri.conf.json: ${tauriVersion}`);
    log.info(`  src-tauri/Cargo.toml:      ${cargoVersion}`);
    return { synced: false, versions: { pkgVersion, tauriVersion, cargoVersion } };
  }

  return { synced: true, version: pkgVersion };
}

// Main function
async function main() {
  const options = parseArgs();

  log.header('Stone Version Tagging');

  // Safety checks
  if (!options.force && !options.noGit) {
    try {
      exec('git rev-parse --git-dir', { silent: true });
    } catch {
      log.error('Not a git repository');
      process.exit(1);
    }

    if (!isWorkingDirectoryClean()) {
      log.error('Working directory has uncommitted changes');
      log.info('Commit or stash your changes first, or use --force to override');
      process.exit(1);
    }

    const branch = getCurrentBranch();
    const mainBranches = ['main', 'master', 'staging', 'develop'];
    const isReleaseBranch = branch.startsWith('release/');
    if (!mainBranches.includes(branch) && !isReleaseBranch) {
      log.warning(`You are on branch '${branch}', not a main or release branch`);
      if (!options.force) {
        log.info('Use --force to continue anyway');
        process.exit(1);
      }
    }
  }

  // Check version sync
  const syncCheck = checkVersionSync();
  if (!syncCheck.synced) {
    if (!options.force) {
      log.error('Version files are out of sync. Use --force to override and set all to the new version.');
      process.exit(1);
    }
    log.warning('Forcing version sync...');
  }

  const currentVersion = syncCheck.synced
    ? syncCheck.version
    : readPackageJson().version;

  if (!semver.valid(currentVersion)) {
    log.error(`Current version '${currentVersion}' is not a valid semver version`);
    process.exit(1);
  }

  log.info(`Current version: ${colors.cyan}${currentVersion}${colors.reset}`);

  // Calculate new version
  const newVersion = calculateNewVersion(currentVersion, options.bumpType, options);

  if (!semver.valid(newVersion)) {
    log.error(`New version '${newVersion}' is not valid`);
    process.exit(1);
  }

  if (currentVersion === newVersion) {
    log.warning('Version unchanged, nothing to do');
    process.exit(0);
  }

  if (!options.force && semver.lt(newVersion, currentVersion)) {
    log.error(`New version ${newVersion} is lower than current version ${currentVersion}`);
    log.info('Use --force to override this check');
    process.exit(1);
  }

  log.info(`New version: ${colors.green}${newVersion}${colors.reset}`);

  const newTag = `v${newVersion}`;
  if (!options.noGit && tagExists(newTag)) {
    log.error(`Tag ${newTag} already exists`);
    if (!options.force) {
      log.info('Use --force to override');
      process.exit(1);
    }
  }

  if (options.dryRun) {
    log.header('DRY RUN MODE - No changes will be made');
  }

  // Update all version files
  log.header('Updating version files...');
  const updates = updateAllVersions(newVersion, options);

  for (const update of updates) {
    log.success(`${update.file}: ${update.oldVersion} -> ${update.newVersion}`);
  }

  // Git operations
  if (!options.noGit) {
    log.header('Committing changes...');
    commitChanges(newVersion, updates, options);
    if (!options.dryRun) {
      log.success('Changes committed');
    }

    log.header('Creating git tag...');
    const tag = createGitTag(newVersion, options);
    if (!options.dryRun) {
      log.success(`Created tag: ${tag}`);
    }

    if (options.push) {
      log.header('Pushing to remote...');
      pushChanges(tag, options);
      if (!options.dryRun) {
        log.success('Changes and tag pushed to remote');
      }
    } else {
      log.info('Not pushing to remote (use --push to push automatically)');
      if (!options.dryRun) {
        const branch = getCurrentBranch();
        log.info(`To push manually: git push origin ${branch} && git push origin ${newTag}`);
      }
    }
  }

  // Summary
  log.header('Summary');
  log.success(`Version bumped: ${currentVersion} -> ${newVersion}`);

  if (options.dryRun) {
    log.warning('This was a dry run - no changes were made');
    log.info('Remove --dry-run flag to apply changes');
  } else {
    log.success('Version tagging completed!');
  }
}

main().catch((error) => {
  log.error(`Error: ${error.message}`);
  if (process.argv.includes('--verbose') || process.argv.includes('-v')) {
    console.error(error.stack);
  }
  process.exit(1);
});

module.exports = { calculateNewVersion, updateAllVersions };
