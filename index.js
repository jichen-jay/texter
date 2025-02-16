import { promises as fs } from "fs";
import path from "path";

function generateRandomString(length) {
  const characters =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  let result = "";

  for (let i = 0; i < length; i++) {
    const randomIndex = Math.floor(Math.random() * characters.length);
    result += characters.charAt(randomIndex);
  }
  return result;
}

async function walkDirectory(directory, recursive) {
  const files = await fs.readdir(directory, { withFileTypes: true });
  let allFiles = [];

  for (const file of files) {
    const fullPath = path.join(directory, file.name);
    if (file.isDirectory() && recursive) {
      const subFiles = await walkDirectory(fullPath, recursive);
      allFiles = allFiles.concat(subFiles);
    } else {
      allFiles.push(fullPath);
    }
  }
  return allFiles;
}

async function combineTextFiles(directory, extension, recursive = false) {
  const files = await walkDirectory(directory, recursive);
  let combinedContent = "";

  for (const filePath of files) {
    if (filePath.endsWith(extension)) {
      const content = await fs.readFile(filePath, "utf8");
      const fileName = path.basename(filePath);
      combinedContent += `\n${fileName}\n`;
      combinedContent += "=".repeat(fileName.length) + "\n\n";
      combinedContent += content + "\n";
    }
  }

  const randomString = generateRandomString(8);
  const outputFileName = `__${randomString}.txt`;
  await fs.writeFile(outputFileName, combinedContent);
}

// Parse command line arguments
const args = process.argv.slice(2);
const recursiveIndex = args.indexOf("-r");
const recursive = recursiveIndex !== -1;

// Remove -r flag from args if present
if (recursive) {
  args.splice(recursiveIndex, 1);
}

const [directory, extension] = args;

if (!directory || !extension) {
  console.error("Usage: node script.js [-r] <directory> <extension>");
  process.exit(1);
}

// Execute the function
combineTextFiles(directory, extension, recursive)
  .then(() => console.log("Files combined successfully"))
  .catch((error) => {
    console.error("Error:", error);
    process.exit(1);
  });
