const fs = require('fs');
const path = require('path');

const standaloneDir = path.join(__dirname, '..', '.next', 'standalone');
const tauriResourceDir = path.join(__dirname, '..', 'src-tauri', 'resources');

// 创建资源目录
if (!fs.existsSync(tauriResourceDir)) {
  fs.mkdirSync(tauriResourceDir, { recursive: true });
}

// 复制 standalone 输出到资源目录
if (fs.existsSync(standaloneDir)) {
  console.log('Copying standalone output to Tauri resources...');
  
  // 复制整个 standalone 目录
  const targetDir = path.join(tauriResourceDir, '.next', 'standalone');
  
  // 删除旧的目标目录
  if (fs.existsSync(targetDir)) {
    fs.rmSync(targetDir, { recursive: true, force: true });
  }
  
  // 创建目标目录
  fs.mkdirSync(path.dirname(targetDir), { recursive: true });
  
  // 递归复制目录
  function copyRecursiveSync(src, dest) {
    const exists = fs.existsSync(src);
    const stats = exists && fs.statSync(src);
    const isDirectory = exists && stats.isDirectory();
    
    if (isDirectory) {
      if (!fs.existsSync(dest)) {
        fs.mkdirSync(dest, { recursive: true });
      }
      fs.readdirSync(src).forEach(childItemName => {
        copyRecursiveSync(
          path.join(src, childItemName),
          path.join(dest, childItemName)
        );
      });
    } else {
      fs.copyFileSync(src, dest);
    }
  }
  
  copyRecursiveSync(standaloneDir, targetDir);
  
  // 同时复制 .next/static 目录（standalone 模式需要）
  const staticSourceDir = path.join(__dirname, '..', '.next', 'static');
  if (fs.existsSync(staticSourceDir)) {
    const staticTargetDir = path.join(tauriResourceDir, '.next', 'static');
    if (fs.existsSync(staticTargetDir)) {
      fs.rmSync(staticTargetDir, { recursive: true, force: true });
    }
    copyRecursiveSync(staticSourceDir, staticTargetDir);
  }
  
  console.log('Standalone output copied successfully!');
} else {
  console.warn('Standalone directory not found. Please run "npm run build" first.');
  process.exit(1);
}

